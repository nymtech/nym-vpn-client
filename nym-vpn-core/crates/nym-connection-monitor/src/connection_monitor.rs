// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use futures::{FutureExt, future::Fuse};
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle, time::Instant};
use tokio_util::sync::CancellationToken;

use nym_common::trace_err_chain;

use super::ConnectionProbe;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to compute next probe time (instant: {0:?}, delay: {1:?})")]
    ComputeNextProbeTime(Instant, Duration),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConnectionEvent {
    /// When last connection evaluation began
    pub start_timestamp: Instant,

    /// When last connection evaluation ended
    pub end_timestamp: Instant,

    /// Connection status with additional metadata
    pub status: ConnectionStatusEvent,
}

/// Describes current connection evaluation.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConnectionStatusEvent {
    /// Connection is viable and able to transmit the probe
    Viable,

    /// Connection is not able to transmit the probe
    /// This could be intermittent. The probe will be re-transmitted.
    IntermittentFailure { retry: u32 },

    /// Connection has failed
    Failed,
}

/// Phase of connection monitor that defines which strategy to use
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Phase {
    /// More aggressive initial probing strategy to determine if connection is viable
    Initial,

    /// Less aggressive monitoring phase to ensure that connection remains viable
    Monitoring,
}

#[derive(Debug, Copy, Clone)]
pub struct TimingConfig {
    /// Probe timeout in the initial phase
    pub initial_probe_timeout: Duration,

    /// Number of probe retransmissions in the initial phase
    pub initial_probe_retry_count: u32,

    /// Probe timeout in the monitoring phase
    pub monitoring_probe_timeout: Duration,

    /// Number of probe retransmissions in the monitoring phase
    pub monitoring_probe_retry_count: u32,

    /// Delay between two successful probe checks in the monitoring phase
    pub probe_periodicity: Duration,
}

impl TimingConfig {
    /// Default timings suitable for mixnet connections
    pub fn mixnet() -> Self {
        TimingConfig {
            initial_probe_timeout: Duration::from_secs(10),
            initial_probe_retry_count: 5,
            monitoring_probe_timeout: Duration::from_secs(10),
            monitoring_probe_retry_count: 5,
            probe_periodicity: Duration::from_secs(60),
        }
    }

    /// Default timings suitable for two-hop connections using ICMP probe
    pub fn two_hop_icmp() -> Self {
        TimingConfig {
            initial_probe_timeout: Duration::from_secs(3),
            initial_probe_retry_count: 5,
            monitoring_probe_timeout: Duration::from_secs(5),
            monitoring_probe_retry_count: 3,
            probe_periodicity: Duration::from_secs(10),
        }
    }

    /// Default timings suitable for two-hop connections using TCP probe
    pub fn two_hop_tcp() -> Self {
        TimingConfig {
            initial_probe_timeout: Duration::from_secs(10),
            initial_probe_retry_count: 5,
            monitoring_probe_timeout: Duration::from_secs(10),
            monitoring_probe_retry_count: 3,
            probe_periodicity: Duration::from_secs(20),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ConnectionStatus {
    /// Connection is undetermined
    Undetermined,

    /// Connection is viable and able to transmit the probe
    Viable,

    /// Connection is considered as failed due to multiple consecutive probe failures
    Failed,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Undetermined => "undetermined",
            Self::Viable => "viable",
            Self::Failed => "failed",
        })
    }
}

/// Internal connection monitor state
#[derive(Debug, Clone)]
struct State {
    /// Phase of connection monitor
    phase: Phase,

    /// Last determined connection status
    status: ConnectionStatus,

    /// Probe retry attempt
    retry: u32,

    /// Timestamp of last successful reply
    last_reply_at: Option<Instant>,

    /// Timestamp of last probe sent
    last_sent_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Initial,
            status: ConnectionStatus::Undetermined,
            retry: 0,
            last_reply_at: None,
            last_sent_at: Instant::now(),
        }
    }
}

impl State {
    /// Increment retry counter
    fn increment_retry(&mut self) {
        if let Some(retry) = self.retry.checked_add(1) {
            self.retry = retry;
        }
    }

    fn timeout(&self, timing_config: &TimingConfig) -> Duration {
        match self.phase {
            Phase::Initial => timing_config.initial_probe_timeout,
            Phase::Monitoring => timing_config.monitoring_probe_timeout,
        }
    }

    /// Maximum number of retries allowed before considering the connection as failed.
    fn max_retry_count(&self, timing_config: &TimingConfig) -> u32 {
        match self.phase {
            Phase::Initial => timing_config.initial_probe_retry_count,
            Phase::Monitoring => timing_config.monitoring_probe_retry_count,
        }
    }
}

pub struct ConnectionMonitor<T>
where
    T: ConnectionProbe + Send + 'static,
{
    probe: T,
    event_tx: UnboundedSender<ConnectionEvent>,
    state: State,
    timing_config: TimingConfig,
    shutdown_token: CancellationToken,
}

impl<T> ConnectionMonitor<T>
where
    T: ConnectionProbe + Send + 'static,
{
    pub fn spawn(
        probe: T,
        timing_config: TimingConfig,
        event_tx: UnboundedSender<ConnectionEvent>,
        shutdown_token: CancellationToken,
    ) -> JoinHandle<Result<(), Error>> {
        let connection_monitor = Self {
            probe,
            event_tx,
            state: State::default(),
            timing_config,
            shutdown_token,
        };

        tokio::spawn(connection_monitor.run())
    }

    async fn run(mut self) -> Result<(), Error> {
        let timeout = self.state.timeout(&self.timing_config);
        tracing::trace!(
            "Sending initial probe with {} ms timeout",
            timeout.as_millis()
        );
        self.state.last_sent_at = Instant::now();
        let probe_task = self.probe.send(timeout).fuse();
        tokio::pin!(probe_task);

        let next_probe_timer = Fuse::terminated();
        tokio::pin!(next_probe_timer);

        let result = loop {
            tokio::select! {
                res = &mut probe_task => {
                    let current_timestamp = Instant::now();

                    // Compute time when to send the next probe
                    let next_probe_at = match res {
                        Ok(()) => {
                            let elapsed = current_timestamp.duration_since(self.state.last_sent_at);
                            tracing::trace!("Probe succeeded in {} ms", elapsed.as_millis());

                            self.state.last_reply_at = Some(current_timestamp);
                            self.state.retry = 0;

                            // Once the first probe succeeds, switch to monitoring phase
                            if self.state.phase == Phase::Initial {
                                self.state.phase = Phase::Monitoring;
                            }

                            self.send_event(ConnectionEvent {
                                status: ConnectionStatusEvent::Viable,
                                start_timestamp: self.state.last_sent_at,
                                end_timestamp: current_timestamp,
                            });

                            self.state.status = ConnectionStatus::Viable;

                            // Since the probe succeeded, send the next one at longer interval
                            let delay = self.timing_config.probe_periodicity;
                            current_timestamp
                                .checked_add(delay)
                                .ok_or(Error::ComputeNextProbeTime(current_timestamp, delay))?
                        }
                        Err(err) => {
                            trace_err_chain!(err);

                            self.state.increment_retry();

                            // Check if retry count has been reached to declare that connection is lost
                            if self.state.retry > self.state.max_retry_count(&self.timing_config) {
                                self.state.status = ConnectionStatus::Failed;
                                self.send_event(ConnectionEvent {
                                    status: ConnectionStatusEvent::Failed,
                                    start_timestamp: self.state.last_sent_at,
                                    end_timestamp: current_timestamp,
                                });
                            } else {
                                self.send_event(ConnectionEvent {
                                    status: ConnectionStatusEvent::IntermittentFailure { retry: self.state.retry },
                                    start_timestamp: self.state.last_sent_at,
                                    end_timestamp: current_timestamp,
                                });
                            }

                            // Re-transmit in equal intervals to avoid the flood in the event of socket failure
                            let delay = self.state.timeout(&self.timing_config);
                            self.state.last_sent_at
                                .checked_add(delay)
                                .ok_or(Error::ComputeNextProbeTime(self.state.last_sent_at, delay))?
                        }
                    };

                    // Elapsed may lapse due to system clock adjustments
                    let elapsed = next_probe_at.checked_duration_since(current_timestamp);
                    if let Some(elapsed) = elapsed && !elapsed.is_zero() {
                        tracing::trace!("Next probe in {} ms", elapsed.as_millis());
                    } else {
                        tracing::trace!("Next probe is now");
                    }
                    next_probe_timer.set(tokio::time::sleep_until(next_probe_at).fuse());
                }
                _ = &mut next_probe_timer => {
                    let timeout = self.state.timeout(&self.timing_config);
                    tracing::trace!(
                        "Sending next probe with {} ms timeout",
                        timeout.as_millis()
                    );

                    self.state.last_sent_at = Instant::now();
                    probe_task.set(self.probe.send(timeout).fuse());
                }
                _ = self.shutdown_token.cancelled() => {
                    tracing::debug!("Connection monitor is cancelled");
                    break Ok(());
                }
            }
        };

        tracing::info!("Exiting connection monitor");
        result
    }

    fn send_event(&self, event: ConnectionEvent) {
        if self.event_tx.send(event).is_err() {
            tracing::error!("Failed to send event");
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tokio::sync::mpsc;
    use tokio_util::sync::DropGuard;

    use super::*;
    use crate::{
        BoxedProbeError,
        mock_probe::{MockProbe, MockProbeError, Outcome},
    };

    const PROBE_RETRY_COUNT: u32 = 3;
    const INITIAL_PROBE_TIMEOUT: Duration = Duration::from_secs(3);
    const PROBE_PERIODICITY: Duration = Duration::from_secs(10);

    #[test]
    fn mock_probe_send_failure_is_not_timeout() {
        let err = BoxedProbeError::from(MockProbeError::SendFailure);
        assert!(err.0.is_send_failure());
        assert!(!err.0.is_timeout());
    }

    #[test]
    fn mock_probe_timeout_is_not_send_failure() {
        let err = BoxedProbeError::from(MockProbeError::Timeout);
        assert!(!err.0.is_send_failure());
        assert!(err.0.is_timeout());
    }

    #[tokio::test(start_paused = true)]
    #[tracing_test::traced_test]
    async fn test_send_failures_count_toward_connection_loss() {
        let probe = MockProbe::repeating(Outcome::SendFailure);
        let (mut event_rx, _drop_guard) = spawn_monitor(probe);

        let events = collect_events(&mut event_rx, (PROBE_RETRY_COUNT + 1) as usize)
            .await
            .into_iter()
            .map(|event| event.status)
            .collect::<Vec<_>>();

        assert_eq!(
            events,
            vec![
                ConnectionStatusEvent::IntermittentFailure { retry: 1 },
                ConnectionStatusEvent::IntermittentFailure { retry: 2 },
                ConnectionStatusEvent::IntermittentFailure { retry: 3 },
                ConnectionStatusEvent::Failed
            ]
        );
    }

    /// Simulated latency of a successful probe.
    const SUCCEEDED_PROBE_LATENCY: Duration = Duration::from_millis(1500);

    // This test simulates the situation where the initial probe keeps failing.
    // It verifies that each subsequent probe happens exactly at the `initial_probe_timeout` interval.
    // This prevents the flood in case of socket or network failures that result into the probe finishing sooner than the specified timeout.
    #[tokio::test(start_paused = true)]
    #[tracing_test::traced_test]
    async fn test_initial_retry_paced_evenly() {
        let probe = MockProbe::repeating(Outcome::SendFailure);
        let (mut event_rx, _drop_guard) = spawn_monitor(probe);

        let events = collect_events(&mut event_rx, (PROBE_RETRY_COUNT + 1) as usize).await;
        let relative_pace = events
            .windows(2)
            .map(|w| w[1].start_timestamp.duration_since(w[0].start_timestamp))
            .collect::<Vec<_>>();

        assert_eq!(
            relative_pace,
            vec![
                INITIAL_PROBE_TIMEOUT,
                INITIAL_PROBE_TIMEOUT,
                INITIAL_PROBE_TIMEOUT
            ]
        )
    }

    #[tokio::test(start_paused = true)]
    #[tracing_test::traced_test]
    async fn test_successful_checks_paced_with_delay() {
        let probe = MockProbe::repeating(Outcome::Succeed {
            after: SUCCEEDED_PROBE_LATENCY,
        });
        let (mut event_rx, _drop_guard) = spawn_monitor(probe);

        let events = collect_events(&mut event_rx, 4).await;
        let relative_pace = events
            .windows(2)
            .map(|w| w[1].start_timestamp.duration_since(w[0].end_timestamp))
            .collect::<Vec<_>>();

        assert_eq!(
            relative_pace,
            vec![PROBE_PERIODICITY, PROBE_PERIODICITY, PROBE_PERIODICITY]
        )
    }

    #[tokio::test(start_paused = true)]
    #[tracing_test::traced_test]
    async fn test_fail_initial_check() {
        let probe = MockProbe::new(
            vec![Outcome::Timeout, Outcome::Timeout, Outcome::Timeout],
            Outcome::Timeout,
        );
        let (mut event_rx, _drop_guard) = spawn_monitor(probe);

        let events = collect_events(&mut event_rx, (PROBE_RETRY_COUNT + 1) as usize)
            .await
            .into_iter()
            .map(|event| event.status)
            .collect::<Vec<_>>();

        assert_eq!(
            events,
            vec![
                ConnectionStatusEvent::IntermittentFailure { retry: 1 },
                ConnectionStatusEvent::IntermittentFailure { retry: 2 },
                ConnectionStatusEvent::IntermittentFailure { retry: 3 },
                ConnectionStatusEvent::Failed
            ]
        );
    }

    #[tokio::test(start_paused = true)]
    #[tracing_test::traced_test]
    async fn test_successful_monitoring_check() {
        let probe = MockProbe::repeating(Outcome::Succeed {
            after: SUCCEEDED_PROBE_LATENCY,
        });
        let (mut event_rx, _drop_guard) = spawn_monitor(probe);

        let events = collect_events(&mut event_rx, 4)
            .await
            .into_iter()
            .map(|event| event.status)
            .collect::<Vec<_>>();

        assert_eq!(
            events,
            vec![
                ConnectionStatusEvent::Viable,
                ConnectionStatusEvent::Viable,
                ConnectionStatusEvent::Viable,
                ConnectionStatusEvent::Viable
            ]
        )
    }

    #[tokio::test(start_paused = true)]
    #[tracing_test::traced_test]
    async fn test_initial_check_recovery() {
        let probe = MockProbe::new(
            vec![
                Outcome::SendFailure,
                Outcome::Timeout,
                Outcome::Succeed {
                    after: SUCCEEDED_PROBE_LATENCY,
                },
            ],
            Outcome::Succeed {
                after: SUCCEEDED_PROBE_LATENCY,
            },
        );
        let (mut event_rx, _drop_guard) = spawn_monitor(probe);

        let events = collect_events(&mut event_rx, 4)
            .await
            .into_iter()
            .map(|event| event.status)
            .collect::<Vec<_>>();

        assert_eq!(
            events,
            vec![
                ConnectionStatusEvent::IntermittentFailure { retry: 1 },
                ConnectionStatusEvent::IntermittentFailure { retry: 2 },
                ConnectionStatusEvent::Viable,
                ConnectionStatusEvent::Viable
            ]
        )
    }

    #[tokio::test(start_paused = true)]
    #[tracing_test::traced_test]
    async fn test_monitoring_check_recovery() {
        let probe = MockProbe::new(
            vec![
                Outcome::Succeed {
                    after: SUCCEEDED_PROBE_LATENCY,
                },
                Outcome::Timeout,
                Outcome::Succeed {
                    after: SUCCEEDED_PROBE_LATENCY,
                },
                Outcome::Succeed {
                    after: SUCCEEDED_PROBE_LATENCY,
                },
            ],
            Outcome::Timeout,
        );
        let (mut event_rx, _drop_guard) = spawn_monitor(probe);

        let events = collect_events(&mut event_rx, 4)
            .await
            .into_iter()
            .map(|event| event.status)
            .collect::<Vec<_>>();

        assert_eq!(
            events,
            vec![
                ConnectionStatusEvent::Viable,
                ConnectionStatusEvent::IntermittentFailure { retry: 1 },
                ConnectionStatusEvent::Viable,
                ConnectionStatusEvent::Viable,
            ]
        )
    }

    fn spawn_monitor(probe: MockProbe) -> (mpsc::UnboundedReceiver<ConnectionEvent>, DropGuard) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let shutdown_token = CancellationToken::new();

        ConnectionMonitor::spawn(probe, test_config(), event_tx, shutdown_token.child_token());

        (event_rx, shutdown_token.drop_guard())
    }

    async fn collect_events(
        event_rx: &mut mpsc::UnboundedReceiver<ConnectionEvent>,
        limit: usize,
    ) -> Vec<ConnectionEvent> {
        let mut events = Vec::new();

        while let Some(event) = event_rx.recv().await {
            events.push(event);
            if events.len() >= limit {
                break;
            }
        }

        events
    }

    fn test_config() -> TimingConfig {
        // timings are not important since tokio timers are being advanced.
        TimingConfig {
            initial_probe_timeout: INITIAL_PROBE_TIMEOUT,
            initial_probe_retry_count: PROBE_RETRY_COUNT,
            monitoring_probe_timeout: Duration::from_secs(5),
            monitoring_probe_retry_count: PROBE_RETRY_COUNT,
            probe_periodicity: PROBE_PERIODICITY,
        }
    }
}
