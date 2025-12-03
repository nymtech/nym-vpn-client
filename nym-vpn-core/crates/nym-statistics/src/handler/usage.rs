// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use rand::distributions::{Alphanumeric, DistString};
use std::{
    cmp::max,
    time::{Duration, Instant},
};
use time::Date;

use crate::{
    commands::ConfigCommand,
    events::UsageEvent,
    storage::{StatsStorage, models::SessionReport},
};

const SESSION_DURATION_BUCKETS_MIN: [i32; 11] = [1, 3, 5, 7, 9, 11, 15, 20, 30, 45, 60];
const SHUTDOWN_ERROR: &str = "SHUTDOWN_REQUESTED";
const OFFLINE_ENDING: &str = "OFFLINE_ENDING";
const RECOVERABLE_ERROR: &str = "RECOVERABLE_ERROR";

pub(crate) struct UsageHandler {
    storage: StatsStorage,
    session_state: SessionState,
}

impl UsageHandler {
    pub(crate) fn new(storage: StatsStorage) -> Self {
        UsageHandler {
            storage,
            session_state: SessionState::NoSession,
        }
    }

    pub async fn handle_command(&mut self, command: ConfigCommand) {
        match command {
            ConfigCommand::DisableCollection => {
                self.session_state = SessionState::NoSession;
                tracing::debug!("UsageHandler : Deleting statistics storage on stats disabling");
                if let Err(e) = self.storage.delete_all().await {
                    tracing::error!("UsageHandler : Failed to delete stored statistics : {e}");
                }
            }
            // Put ourselves in standby until a session properly starts
            ConfigCommand::EnableCollection => {
                // Avoiding double enable collection overwriting
                if self.session_state == SessionState::NoSession {
                    self.session_state = SessionState::Standby;
                }
            }

            // We don't care about this one
            ConfigCommand::AllowDirectSending(_) => {}
        }
    }

    pub(crate) async fn on_shutdown(&mut self) {
        if let Some(finished_session) = self.session_state.clone().into_shutdown_finished() {
            self.store_session(finished_session).await;
        }
        // Useless because we shutting down but keep it consistent
        self.session_state = SessionState::NoSession;
    }

    pub(crate) async fn handle_event(&mut self, event: UsageEvent) {
        match event {
            // User pressed "Connect"
            UsageEvent::ConnectRequest(instant) => {
                self.handle_connect_request(instant);
            }

            // State machine entered "Connecting" state
            UsageEvent::Connecting {
                instant,
                retry_attempt,
                maybe_exit_id,
                maybe_exit_cc,
                maybe_tunnel_type,
            } => {
                self.handle_connecting(
                    instant,
                    retry_attempt,
                    maybe_exit_id,
                    maybe_exit_cc,
                    maybe_tunnel_type,
                )
                .await;
            }

            // State machine entered "Connected" state
            UsageEvent::Connected {
                instant,
                exit_id,
                exit_cc,
                tunnel_type,
            } => {
                self.handle_connected(instant, exit_id, exit_cc, tunnel_type);
            }

            // User pressed "Disconnect"
            UsageEvent::DisconnectRequest(instant) => {
                self.handle_disconnect_request(instant);
            }

            // State machine entered "Disconnecting" state
            UsageEvent::Disconnecting { instant, .. } => {
                self.handle_disconnecting(instant);
            }

            // State machine entered "Disconnected" state
            UsageEvent::Disconnected(instant) => {
                self.handle_disconnected(instant).await;
            }

            // State machine entered "Error" state
            UsageEvent::Error { instant, error } => {
                self.handle_error(instant, error).await;
            }

            // State machine entered "Offline" state
            UsageEvent::Offline { reconnect, instant } => {
                self.handle_offline(instant, reconnect).await
            }
        }
    }

    async fn store_session(&mut self, session: FinishedSession) {
        if let Err(e) = self
            .storage
            .insert_pending_session_report(&session.into())
            .await
        {
            tracing::warn!("Failed to store session report : {e}")
        }
    }

    fn handle_connect_request(&mut self, start_time: Instant) {
        match self.session_state {
            // We are disconnected, and we want to connect
            SessionState::NoSession | SessionState::Standby => {
                self.session_state = SessionState::ConnectionRequested {
                    start_day: time::OffsetDateTime::now_utc().date(),
                    start_time,
                }
            }
            // We were connecting, and `reconnect` was called. Let's just scrap the current session
            SessionState::Connecting { .. } => {
                self.session_state = SessionState::ConnectionRequested {
                    start_day: time::OffsetDateTime::now_utc().date(),
                    start_time,
                }
            }
            // We are offline, we don't do anything, as to not count the offline time as connecting time
            SessionState::Offline => {}
            // We are connected (or disconnecting), and `reconnect` (or connect) was called. TSM states will finish the session
            SessionState::OnGoing { .. } | SessionState::Disconnecting { .. } => {}
            _ => {
                tracing::warn!(
                    "UsageHandler: Received connect request event while in state {}",
                    self.session_state
                );
            }
        }
    }

    async fn handle_connecting(
        &mut self,
        event_time: Instant,
        new_retry_attempt: u32,
        maybe_exit_id: Option<String>,
        maybe_exit_cc: Option<String>,
        maybe_tunnel_type: Option<String>,
    ) {
        match self.session_state.clone() {
            // Connection was requested, we were not offline
            SessionState::ConnectionRequested {
                start_day,
                start_time,
            } => {
                self.session_state = SessionState::Connecting {
                    start_day,
                    start_time,
                    retry_attempt: new_retry_attempt,
                    tunnel_type: maybe_tunnel_type,
                    exit_id: maybe_exit_id,
                    exit_cc: maybe_exit_cc,
                    follow_up_id: None,
                }
            }
            // Connection was requested when we were offline, we want to only capture the actual connecting time
            // Or stats were enabled somewhere in the middle
            SessionState::Offline | SessionState::Standby => {
                self.session_state = SessionState::Connecting {
                    start_day: time::OffsetDateTime::now_utc().date(),
                    start_time: event_time,
                    retry_attempt: new_retry_attempt,
                    tunnel_type: maybe_tunnel_type,
                    exit_id: maybe_exit_id,
                    exit_cc: maybe_exit_cc,
                    follow_up_id: None,
                }
            }
            // Connection was normally requested but the connection request event wasn't fired
            // This shouldn't happen and should be fixed, by properly firing the event
            // In the meantime, let's still start the session
            SessionState::NoSession => {
                tracing::warn!(
                    "Started a Session without a connecting request. This shouldn't happen"
                );
                self.session_state = SessionState::Connecting {
                    start_day: time::OffsetDateTime::now_utc().date(),
                    start_time: event_time,
                    retry_attempt: new_retry_attempt,
                    tunnel_type: maybe_tunnel_type,
                    exit_id: maybe_exit_id,
                    exit_cc: maybe_exit_cc,
                    follow_up_id: None,
                }
            }
            // Connecting state is entered multiple times, let's adjust the retries and connection data if needed
            SessionState::Connecting {
                start_day,
                start_time,
                retry_attempt,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => {
                self.session_state = SessionState::Connecting {
                    start_day,
                    start_time,
                    retry_attempt: max(retry_attempt, new_retry_attempt),
                    tunnel_type: maybe_tunnel_type.or(tunnel_type), // update with the new value, maybe further attempts changed it
                    exit_id: maybe_exit_id.or(exit_id), // update with the new value, maybe further attempts changed it
                    exit_cc: maybe_exit_cc.or(exit_cc),
                    follow_up_id,
                }
            }
            // Connection hiccup, store session and link the next one to it
            SessionState::OnGoing {
                start_day,
                connection_duration,
                retry_attempt,
                session_start_time,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => {
                let session_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 20);
                let finished_session = FinishedSession {
                    start_day,
                    connection_duration,
                    retry_attempt,
                    session_duration: event_time.duration_since(session_start_time),
                    disconnection_duration: Duration::from_millis(0),
                    tunnel_type,
                    exit_id,
                    exit_cc,
                    follow_up_id,
                    error: Some(format!("{RECOVERABLE_ERROR}_{session_id}")),
                };
                self.store_session(finished_session).await;
                self.session_state = SessionState::Connecting {
                    start_day: time::OffsetDateTime::now_utc().date(),
                    start_time: event_time,
                    retry_attempt: new_retry_attempt,
                    tunnel_type: maybe_tunnel_type,
                    exit_id: maybe_exit_id,
                    exit_cc: maybe_exit_cc,
                    follow_up_id: Some(session_id),
                }
            }

            // We were disconnecting and connecting right after it
            // Happens on settings change and `connect` while disconnecting
            // Store the session and start a new one
            SessionState::Disconnecting {
                start_day,
                connection_duration,
                retry_attempt,
                session_duration,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
                disconnecting_time,
            } => {
                let finished_session = FinishedSession {
                    start_day,
                    connection_duration,
                    retry_attempt,
                    session_duration,
                    disconnection_duration: event_time.duration_since(disconnecting_time),
                    tunnel_type,
                    exit_id,
                    exit_cc,
                    follow_up_id,
                    error: None,
                };
                self.store_session(finished_session).await;
                self.session_state = SessionState::Connecting {
                    start_day: time::OffsetDateTime::now_utc().date(),
                    start_time: event_time,
                    retry_attempt: new_retry_attempt,
                    tunnel_type: maybe_tunnel_type,
                    exit_id: maybe_exit_id,
                    exit_cc: maybe_exit_cc,
                    follow_up_id: None,
                }
            }
        }
    }

    fn handle_connected(
        &mut self,
        session_start_time: Instant,
        exit_id: String,
        exit_cc: Option<String>,
        tunnel_type: String,
    ) {
        match self.session_state.clone() {
            SessionState::Connecting {
                start_day,
                start_time,
                retry_attempt,
                follow_up_id,
                ..
            } => {
                self.session_state = SessionState::OnGoing {
                    start_day,
                    connection_duration: session_start_time.duration_since(start_time),
                    retry_attempt,
                    session_start_time,
                    tunnel_type,
                    exit_id,
                    exit_cc,
                    follow_up_id,
                }
            }
            // Stats were enabled during a session
            SessionState::Standby => {}
            _ => {
                tracing::warn!(
                    "UsageHandler: Received connected event while in state {}",
                    self.session_state
                );
            }
        }
    }

    fn handle_disconnect_request(&mut self, session_disconnection_start: Instant) {
        match self.session_state.clone() {
            // Happy path
            SessionState::OnGoing {
                start_day,
                connection_duration,
                retry_attempt,
                session_start_time,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => {
                self.session_state = SessionState::Disconnecting {
                    start_day,
                    connection_duration,
                    retry_attempt,
                    session_duration: session_disconnection_start
                        .duration_since(session_start_time),
                    tunnel_type,
                    exit_id,
                    exit_cc,
                    follow_up_id,
                    disconnecting_time: session_disconnection_start,
                }
            }
            // Disconnection was requested while we were connecting
            SessionState::Connecting {
                start_day,
                start_time,
                retry_attempt,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => {
                self.session_state = SessionState::Disconnecting {
                    start_day,
                    connection_duration: session_disconnection_start.duration_since(start_time),
                    retry_attempt,
                    session_duration: Duration::from_millis(0),
                    tunnel_type: tunnel_type.unwrap_or_default(),
                    exit_id: exit_id.unwrap_or_default(),
                    exit_cc,
                    follow_up_id,
                    disconnecting_time: session_disconnection_start,
                }
            }
            SessionState::ConnectionRequested { .. } => {
                // This shouldn't happen but anyway
                self.session_state = SessionState::NoSession
            }

            // Happens if connection request and disconnection request while offline
            SessionState::Offline => {}

            // Stats were enabled during a session
            SessionState::Standby => {}

            _ => {
                tracing::warn!(
                    "UsageHandler: Received disconnect request event while in state {}",
                    self.session_state
                );
            }
        }
    }

    fn handle_disconnecting(&mut self, event_time: Instant) {
        match self.session_state.clone() {
            // Errors in connecting state can lead to here directly
            SessionState::Connecting {
                start_day,
                start_time,
                retry_attempt,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => {
                self.session_state = SessionState::Disconnecting {
                    start_day,
                    connection_duration: event_time.duration_since(start_time),
                    retry_attempt,
                    session_duration: Duration::from_millis(0),
                    tunnel_type: tunnel_type.unwrap_or_default(),
                    exit_id: exit_id.unwrap_or_default(),
                    exit_cc,
                    follow_up_id,
                    disconnecting_time: event_time,
                }
            }
            // Disconnetion, either because of error, setting changes or offline
            SessionState::OnGoing {
                start_day,
                connection_duration,
                retry_attempt,
                session_start_time,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => {
                self.session_state = SessionState::Disconnecting {
                    start_day,
                    connection_duration,
                    retry_attempt,
                    session_duration: event_time.duration_since(session_start_time),
                    tunnel_type,
                    exit_id,
                    exit_cc,
                    follow_up_id,
                    disconnecting_time: event_time,
                }
            }
            // Entering disconnecting state after a disconnection request
            SessionState::Disconnecting { .. } => {}

            // Stats were enabled during a session
            SessionState::Standby => {}
            _ => {
                tracing::warn!(
                    "UsageHandler: Received disconnecting event while in state {}",
                    self.session_state
                );
            }
        }
    }

    async fn handle_disconnected(&mut self, session_end_time: Instant) {
        match self.session_state.clone() {
            SessionState::Disconnecting {
                start_day,
                connection_duration,
                retry_attempt,
                session_duration,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
                disconnecting_time,
            } => {
                let finished_session = FinishedSession {
                    start_day,
                    connection_duration,
                    retry_attempt,
                    session_duration,
                    disconnection_duration: session_end_time.duration_since(disconnecting_time),
                    tunnel_type,
                    exit_id,
                    exit_cc,
                    follow_up_id,
                    error: None,
                };
                self.store_session(finished_session).await;
                self.session_state = SessionState::NoSession;
            }
            SessionState::NoSession => {}

            SessionState::Offline | SessionState::Standby => {
                self.session_state = SessionState::NoSession
            }
            _ => {
                tracing::warn!(
                    "UsageHandler: Received disconnected event while in state {}",
                    self.session_state
                );
            }
        }
    }

    async fn handle_error(&mut self, error_time: Instant, error_string: String) {
        match self.session_state.clone() {
            SessionState::OnGoing {
                start_day,
                connection_duration,
                retry_attempt,
                session_start_time,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => {
                let finished_session = FinishedSession {
                    start_day,
                    connection_duration,
                    retry_attempt,
                    session_duration: error_time.duration_since(session_start_time),
                    disconnection_duration: Duration::from_millis(0),
                    tunnel_type,
                    exit_id,
                    exit_cc,
                    follow_up_id,
                    error: Some(error_string),
                };
                self.store_session(finished_session).await;
                self.session_state = SessionState::NoSession;
            }
            SessionState::Connecting {
                start_day,
                start_time,
                retry_attempt,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => {
                let finished_session = FinishedSession {
                    start_day,
                    connection_duration: error_time.duration_since(start_time),
                    retry_attempt,
                    session_duration: Duration::from_millis(0),
                    disconnection_duration: Duration::from_millis(0),
                    tunnel_type: tunnel_type.unwrap_or_default(),
                    exit_id: exit_id.unwrap_or_default(),
                    exit_cc,
                    follow_up_id,
                    error: Some(error_string),
                };
                self.store_session(finished_session).await;
                self.session_state = SessionState::NoSession;
            }
            SessionState::Disconnecting {
                start_day,
                connection_duration,
                retry_attempt,
                session_duration,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
                disconnecting_time,
            } => {
                let finished_session = FinishedSession {
                    start_day,
                    connection_duration,
                    retry_attempt,
                    session_duration,
                    disconnection_duration: error_time.duration_since(disconnecting_time),
                    tunnel_type,
                    exit_id,
                    exit_cc,
                    follow_up_id,
                    error: Some(error_string),
                };
                self.store_session(finished_session).await;
                self.session_state = SessionState::NoSession;
            }
            SessionState::Standby => self.session_state = SessionState::NoSession,
            _ => {
                tracing::warn!(
                    "UsageHandler: Received error event while in state {}",
                    self.session_state
                );
            }
        }
    }

    async fn handle_offline(&mut self, event_time: Instant, reconnect: bool) {
        match self.session_state.clone() {
            // Helps to properly count connection request while offline
            SessionState::NoSession if !reconnect => {
                self.session_state = SessionState::Offline;
            }

            SessionState::Standby => self.session_state = SessionState::Offline,

            // Connection request while offline, no-op
            // Also happens if disconnection request
            SessionState::Offline => {}

            // Handle disconnection requestion while offline
            SessionState::Disconnecting {
                start_day,
                connection_duration,
                retry_attempt,
                session_duration,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
                disconnecting_time,
            } => {
                let finished_session = FinishedSession {
                    start_day,
                    connection_duration,
                    retry_attempt,
                    session_duration,
                    disconnection_duration: event_time.duration_since(disconnecting_time),
                    tunnel_type,
                    exit_id,
                    exit_cc,
                    follow_up_id,
                    error: Some(OFFLINE_ENDING.into()),
                };
                self.store_session(finished_session).await;
                self.session_state = SessionState::Offline;
            }
            SessionState::Connecting {
                start_day,
                start_time,
                retry_attempt,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => {
                let finished_session = FinishedSession {
                    start_day,
                    connection_duration: event_time.duration_since(start_time),
                    retry_attempt,
                    session_duration: Duration::from_millis(0),
                    disconnection_duration: Duration::from_millis(0),
                    tunnel_type: tunnel_type.unwrap_or_default(),
                    exit_id: exit_id.unwrap_or_default(),
                    exit_cc,
                    follow_up_id,
                    error: Some(OFFLINE_ENDING.into()),
                };
                self.store_session(finished_session).await;
                self.session_state = SessionState::Offline;
            }
            _ => {
                tracing::warn!(
                    "UsageHandler: Received offline event while in state {}",
                    self.session_state
                );
            }
        }
    }
}

fn bucketize_session_duration(session_duration_secs: u64) -> i32 {
    // Special case for sessions that couldn't start
    if session_duration_secs == 0 {
        return 0;
    }
    let session_duration_min = (session_duration_secs / 60).try_into().unwrap_or(i32::MAX);
    for upper_bound in SESSION_DURATION_BUCKETS_MIN {
        if session_duration_min < upper_bound {
            return upper_bound;
        }
    }
    (session_duration_min / SESSION_DURATION_BUCKETS_MIN[SESSION_DURATION_BUCKETS_MIN.len() - 1]
        + 1)
        * SESSION_DURATION_BUCKETS_MIN[SESSION_DURATION_BUCKETS_MIN.len() - 1] // Round to next multiple of last bound
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct FinishedSession {
    start_day: Date,
    connection_duration: Duration,
    retry_attempt: u32,
    session_duration: Duration,
    tunnel_type: String,
    exit_id: String,
    exit_cc: Option<String>,
    follow_up_id: Option<String>,
    disconnection_duration: Duration,
    error: Option<String>,
}

impl From<FinishedSession> for SessionReport {
    fn from(value: FinishedSession) -> Self {
        let connection_time_ms = value
            .connection_duration
            .as_millis()
            .try_into()
            .unwrap_or(i32::MAX);
        let disconnection_time_ms = value
            .disconnection_duration
            .as_millis()
            .try_into()
            .unwrap_or(i32::MAX);
        let session_duration_min = bucketize_session_duration(value.session_duration.as_secs());
        let retry_attempt = value.retry_attempt.try_into().unwrap_or(i32::MAX);
        SessionReport {
            day_utc: value.start_day,
            connection_time_ms,
            retry_attempt,
            session_duration_min,
            disconnection_time_ms,
            tunnel_type: value.tunnel_type,
            exit_id: value.exit_id,
            exit_cc: value.exit_cc,
            follow_up_id: value.follow_up_id,
            error: value.error,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, strum::Display)]
enum SessionState {
    // Stats were enabled while we were in a session
    Standby,

    // Nothing is going on
    NoSession,

    // We are offline
    Offline,

    // Connection was requested, but we aren't connecting yet
    ConnectionRequested {
        start_day: Date,
        start_time: Instant,
    },

    // We are connecting
    Connecting {
        start_day: Date,
        start_time: Instant,
        retry_attempt: u32,
        tunnel_type: Option<String>,
        exit_id: Option<String>,
        exit_cc: Option<String>,
        follow_up_id: Option<String>,
    },

    // We are connected
    OnGoing {
        start_day: Date,
        connection_duration: Duration,
        retry_attempt: u32,
        session_start_time: Instant,
        tunnel_type: String,
        exit_id: String,
        exit_cc: Option<String>,
        follow_up_id: Option<String>,
    },

    // We are disconnected
    Disconnecting {
        start_day: Date,
        connection_duration: Duration,
        retry_attempt: u32,
        session_duration: Duration,
        tunnel_type: String,
        exit_id: String,
        exit_cc: Option<String>,
        follow_up_id: Option<String>,
        disconnecting_time: Instant,
    },
}

impl SessionState {
    fn into_shutdown_finished(self) -> Option<FinishedSession> {
        match self {
            SessionState::NoSession => None,
            Self::Standby => None,
            SessionState::Offline => None,
            SessionState::ConnectionRequested { .. } => None,
            SessionState::Connecting {
                start_day,
                start_time,
                retry_attempt,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => Some(FinishedSession {
                start_day,
                connection_duration: Instant::now().duration_since(start_time),
                retry_attempt,
                session_duration: Duration::from_millis(0),
                disconnection_duration: Duration::from_millis(0),
                tunnel_type: tunnel_type.unwrap_or_default(),
                exit_id: exit_id.unwrap_or_default(),
                exit_cc,
                follow_up_id,
                error: Some(SHUTDOWN_ERROR.into()),
            }),
            SessionState::OnGoing {
                start_day,
                connection_duration,
                retry_attempt,
                session_start_time,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
            } => Some(FinishedSession {
                start_day,
                connection_duration,
                retry_attempt,
                session_duration: Instant::now().duration_since(session_start_time),
                disconnection_duration: Duration::from_millis(0),
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
                error: Some(SHUTDOWN_ERROR.into()),
            }),
            SessionState::Disconnecting {
                start_day,
                connection_duration,
                retry_attempt,
                session_duration,
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
                disconnecting_time,
            } => Some(FinishedSession {
                start_day,
                connection_duration,
                retry_attempt,
                session_duration,
                disconnection_duration: Instant::now().duration_since(disconnecting_time),
                tunnel_type,
                exit_id,
                exit_cc,
                follow_up_id,
                error: Some(SHUTDOWN_ERROR.into()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use futures::executor::block_on;
    use nym_vpn_lib_types::{ActionAfterDisconnect, TunnelType};

    use super::*;
    use crate::events::UsageEvent;

    fn mock_usage_handler() -> UsageHandler {
        let mock_storage = block_on(crate::storage::test::mock_database());
        UsageHandler::new(mock_storage)
    }
    fn mock_gateway_id_a() -> String {
        "gatewayA".into()
    }
    fn mock_gateway_cc_a() -> String {
        "EU".into()
    }
    fn mock_gateway_id_b() -> String {
        "gatewayB".into()
    }
    fn mock_gateway_cc_b() -> String {
        "DS".into()
    }
    fn wg_tunnel_type() -> String {
        TunnelType::Wireguard.short_name().into()
    }

    impl UsageHandler {
        fn assert_stored_session(&self, expected: SessionReport) {
            let stored_sessions =
                block_on(self.storage.get_pending_session_report_with_id()).unwrap();
            let stored_session = stored_sessions.first().unwrap();

            assert_eq!(expected, stored_session.report);
        }
    }

    #[tokio::test]
    async fn successfull_session_test() {
        // Mostly happy path, with some variant in connecting state
        let mut usage_handler = mock_usage_handler();

        // Time info
        let start_day = time::OffsetDateTime::now_utc().date();
        let connect_request_instant = Instant::now();
        let connection_duration = Duration::from_millis(1234);
        let connected_instant = connect_request_instant + connection_duration;

        let session_duration = Duration::from_secs(123);
        let disconnect_request_instant = connected_instant + session_duration;

        let disconnection_duration = Duration::from_millis(1234);
        let disconnected_instant = disconnect_request_instant + disconnection_duration;

        // Connection request
        usage_handler
            .handle_event(UsageEvent::ConnectRequest(connect_request_instant))
            .await;
        assert_eq!(
            usage_handler.session_state,
            SessionState::ConnectionRequested {
                start_day,
                start_time: connect_request_instant,
            }
        );

        // Connecting events
        usage_handler
            .handle_event(UsageEvent::Connecting {
                instant: Instant::now(),
                retry_attempt: 0,
                maybe_exit_id: None,
                maybe_exit_cc: None,
                maybe_tunnel_type: None,
            })
            .await;
        assert_eq!(
            usage_handler.session_state,
            SessionState::Connecting {
                start_day,
                start_time: connect_request_instant,
                retry_attempt: 0,
                tunnel_type: None,
                exit_id: None,
                exit_cc: None,
                follow_up_id: None,
            }
        );

        // Info update
        usage_handler
            .handle_event(UsageEvent::Connecting {
                instant: Instant::now(),
                retry_attempt: 1,
                maybe_exit_id: Some(mock_gateway_id_a()),
                maybe_exit_cc: Some(mock_gateway_cc_a()),
                maybe_tunnel_type: Some(wg_tunnel_type()),
            })
            .await;
        assert_eq!(
            usage_handler.session_state,
            SessionState::Connecting {
                start_day,
                start_time: connect_request_instant,
                retry_attempt: 1,
                exit_id: Some(mock_gateway_id_a()),
                exit_cc: Some(mock_gateway_cc_a()),
                tunnel_type: Some(wg_tunnel_type()),
                follow_up_id: None,
            }
        );

        // Nothing gets wrongly overwritten (even though it shouldn't happened)
        usage_handler
            .handle_event(UsageEvent::Connecting {
                instant: Instant::now(),
                retry_attempt: 0,
                maybe_exit_id: None,
                maybe_exit_cc: None,
                maybe_tunnel_type: None,
            })
            .await;
        assert_eq!(
            usage_handler.session_state,
            SessionState::Connecting {
                start_day,
                start_time: connect_request_instant,
                retry_attempt: 1,
                exit_id: Some(mock_gateway_id_a()),
                exit_cc: Some(mock_gateway_cc_a()),
                tunnel_type: Some(wg_tunnel_type()),
                follow_up_id: None,
            }
        );

        // If for some reason exit is updated we're updating it
        usage_handler
            .handle_event(UsageEvent::Connecting {
                instant: Instant::now(),
                retry_attempt: 1,
                maybe_exit_id: Some(mock_gateway_id_b()),
                maybe_exit_cc: Some(mock_gateway_cc_b()),
                maybe_tunnel_type: Some(wg_tunnel_type()),
            })
            .await;
        assert_eq!(
            usage_handler.session_state,
            SessionState::Connecting {
                start_day,
                start_time: connect_request_instant,
                retry_attempt: 1,
                exit_id: Some(mock_gateway_id_b()),
                exit_cc: Some(mock_gateway_cc_b()),
                tunnel_type: Some(wg_tunnel_type()),
                follow_up_id: None,
            }
        );

        // Connected
        usage_handler
            .handle_event(UsageEvent::Connected {
                instant: connected_instant,
                exit_id: mock_gateway_id_b(),
                exit_cc: Some(mock_gateway_cc_b()),
                tunnel_type: wg_tunnel_type(),
            })
            .await;

        assert_eq!(
            usage_handler.session_state,
            SessionState::OnGoing {
                start_day,
                connection_duration,
                retry_attempt: 1,
                session_start_time: connected_instant,
                tunnel_type: wg_tunnel_type(),
                exit_id: mock_gateway_id_b(),
                exit_cc: Some(mock_gateway_cc_b()),
                follow_up_id: None,
            }
        );

        // Disconnection request
        usage_handler
            .handle_event(UsageEvent::DisconnectRequest(disconnect_request_instant))
            .await;

        assert_eq!(
            usage_handler.session_state,
            SessionState::Disconnecting {
                start_day,
                connection_duration,
                retry_attempt: 1,
                session_duration,
                tunnel_type: wg_tunnel_type(),
                exit_id: mock_gateway_id_b(),
                exit_cc: Some(mock_gateway_cc_b()),
                disconnecting_time: disconnect_request_instant,
                follow_up_id: None,
            }
        );

        // Disconnecting state
        usage_handler
            .handle_event(UsageEvent::Disconnecting {
                instant: Instant::now(),
                after_disconnect: ActionAfterDisconnect::Nothing,
            })
            .await;
        assert_eq!(
            usage_handler.session_state,
            SessionState::Disconnecting {
                start_day,
                connection_duration,
                retry_attempt: 1,
                session_duration,
                tunnel_type: wg_tunnel_type(),
                exit_id: mock_gateway_id_b(),
                exit_cc: Some(mock_gateway_cc_b()),
                disconnecting_time: disconnect_request_instant,
                follow_up_id: None,
            }
        );

        // Disconnected and session end
        usage_handler
            .handle_event(UsageEvent::Disconnected(disconnected_instant))
            .await;
        assert_eq!(usage_handler.session_state, SessionState::NoSession);

        // Test that storage is correct
        let expected_report = SessionReport {
            day_utc: start_day,
            connection_time_ms: connection_duration.as_millis().try_into().unwrap(),
            retry_attempt: 1,
            session_duration_min: bucketize_session_duration(session_duration.as_secs()),
            tunnel_type: wg_tunnel_type(),
            exit_id: mock_gateway_id_b(),
            exit_cc: Some(mock_gateway_cc_b()),
            follow_up_id: None,
            error: None,
            disconnection_time_ms: disconnection_duration.as_millis().try_into().unwrap(),
        };

        usage_handler.assert_stored_session(expected_report);
    }

    #[tokio::test]
    async fn irrecoverable_tunnel_error() {
        let mut usage_handler = mock_usage_handler();

        // Time info
        let start_day = time::OffsetDateTime::now_utc().date();
        let connect_request_instant = Instant::now();
        let connection_duration = Duration::from_millis(1234);
        let connected_instant = connect_request_instant + connection_duration;

        let session_duration = Duration::from_secs(123);
        let error_instant = connected_instant + session_duration;
        let error_reason = "I suppose you think that was terribly clever";

        // Connection info
        let retry_attempt = 0;

        // Start connected for convienience
        usage_handler.session_state = SessionState::OnGoing {
            start_day,
            connection_duration,
            retry_attempt,
            session_start_time: connected_instant,
            tunnel_type: wg_tunnel_type(),
            exit_id: mock_gateway_id_a(),
            exit_cc: Some(mock_gateway_cc_a()),
            follow_up_id: None,
        };

        // Abrupt irrecoverable error
        usage_handler
            .handle_event(UsageEvent::Error {
                instant: error_instant,
                error: error_reason.into(),
            })
            .await;

        assert_eq!(usage_handler.session_state, SessionState::NoSession);

        let expected_report = SessionReport {
            day_utc: start_day,
            connection_time_ms: connection_duration.as_millis().try_into().unwrap(),
            retry_attempt: 0,
            session_duration_min: bucketize_session_duration(session_duration.as_secs()),
            disconnection_time_ms: 0,
            tunnel_type: wg_tunnel_type(),
            exit_id: mock_gateway_id_a(),
            exit_cc: Some(mock_gateway_cc_a()),
            follow_up_id: None,
            error: Some(error_reason.into()),
        };

        usage_handler.assert_stored_session(expected_report);
    }

    #[tokio::test]
    async fn settings_change() {
        let mut usage_handler = mock_usage_handler();

        // Time info
        let start_day = time::OffsetDateTime::now_utc().date();
        let connect_request_instant = Instant::now();
        let connection_duration = Duration::from_millis(1234);
        let connected_instant = connect_request_instant + connection_duration;

        let session_duration = Duration::from_secs(123);
        let disconnecting_instant = connected_instant + session_duration;

        let disconnection_duration = Duration::from_millis(2345);
        let reconnecting_instant = disconnecting_instant + disconnection_duration;

        let reconnection_duration = Duration::from_millis(5678);
        let reconnected_instant = reconnecting_instant + reconnection_duration;

        // Connection info
        let retry_attempt = 0;

        // Start connected for convienience
        usage_handler.session_state = SessionState::OnGoing {
            start_day,
            connection_duration,
            retry_attempt,
            session_start_time: connected_instant,
            tunnel_type: wg_tunnel_type(),
            exit_id: mock_gateway_id_a(),
            exit_cc: Some(mock_gateway_cc_a()),
            follow_up_id: None,
        };

        // Settings changed, disconnecting to reconnect
        usage_handler
            .handle_event(UsageEvent::Disconnecting {
                instant: disconnecting_instant,
                after_disconnect: ActionAfterDisconnect::Reconnect,
            })
            .await;

        assert_eq!(
            usage_handler.session_state,
            SessionState::Disconnecting {
                start_day,
                connection_duration,
                retry_attempt,
                session_duration,
                tunnel_type: wg_tunnel_type(),
                exit_id: mock_gateway_id_a(),
                exit_cc: Some(mock_gateway_cc_a()),
                disconnecting_time: disconnecting_instant,
                follow_up_id: None,
            }
        );

        // Connecting with the new settings
        usage_handler
            .handle_event(UsageEvent::Connecting {
                instant: reconnecting_instant,
                retry_attempt,
                maybe_exit_id: None,
                maybe_exit_cc: None,
                maybe_tunnel_type: None,
            })
            .await;

        assert_eq!(
            usage_handler.session_state,
            SessionState::Connecting {
                start_day,
                start_time: reconnecting_instant,
                retry_attempt,
                tunnel_type: None,
                exit_id: None,
                exit_cc: None,
                follow_up_id: None,
            }
        );

        usage_handler
            .handle_event(UsageEvent::Connected {
                instant: reconnected_instant,
                tunnel_type: wg_tunnel_type(),
                exit_id: mock_gateway_id_b(),
                exit_cc: Some(mock_gateway_cc_b()),
            })
            .await;

        assert_eq!(
            usage_handler.session_state,
            SessionState::OnGoing {
                start_day,
                connection_duration: reconnection_duration,
                retry_attempt,
                session_start_time: reconnected_instant,
                tunnel_type: wg_tunnel_type(),
                exit_id: mock_gateway_id_b(),
                exit_cc: Some(mock_gateway_cc_b()),
                follow_up_id: None,
            }
        );

        let expected_report = SessionReport {
            day_utc: start_day,
            connection_time_ms: connection_duration.as_millis().try_into().unwrap(),
            retry_attempt: 0,
            session_duration_min: bucketize_session_duration(session_duration.as_secs()),
            disconnection_time_ms: disconnection_duration.as_millis().try_into().unwrap(),
            tunnel_type: wg_tunnel_type(),
            exit_id: mock_gateway_id_a(),
            exit_cc: Some(mock_gateway_cc_a()),
            follow_up_id: None,
            error: None,
        };

        usage_handler.assert_stored_session(expected_report);
    }

    #[tokio::test]
    async fn disable_enable_stats_during_session() {
        // Mostly happy path, with some variant in connecting state
        let mut usage_handler = mock_usage_handler();

        // Time info
        let start_day = time::OffsetDateTime::now_utc().date();
        let connect_request_instant = Instant::now();
        let connection_duration = Duration::from_millis(1234);
        let connected_instant = connect_request_instant + connection_duration;

        let session_duration = Duration::from_secs(123);
        let disconnect_request_instant = connected_instant + session_duration;

        let disconnection_duration = Duration::from_millis(1234);
        let disconnected_instant = disconnect_request_instant + disconnection_duration;

        let retry_attempt = 0;

        // Start connected for convienience
        usage_handler.session_state = SessionState::OnGoing {
            start_day,
            connection_duration,
            retry_attempt,
            session_start_time: connected_instant,
            tunnel_type: wg_tunnel_type(),
            exit_id: mock_gateway_id_a(),
            exit_cc: Some(mock_gateway_cc_a()),
            follow_up_id: None,
        };

        usage_handler
            .handle_command(ConfigCommand::DisableCollection)
            .await;
        assert_eq!(usage_handler.session_state, SessionState::NoSession);

        usage_handler
            .handle_command(ConfigCommand::EnableCollection)
            .await;
        assert_eq!(usage_handler.session_state, SessionState::Standby);

        // Disconnection request
        usage_handler
            .handle_event(UsageEvent::DisconnectRequest(disconnect_request_instant))
            .await;

        assert_eq!(usage_handler.session_state, SessionState::Standby);

        // Disconnecting state
        usage_handler
            .handle_event(UsageEvent::Disconnecting {
                instant: Instant::now(),
                after_disconnect: ActionAfterDisconnect::Nothing,
            })
            .await;

        assert_eq!(usage_handler.session_state, SessionState::Standby);

        // Disconnected and session end
        usage_handler
            .handle_event(UsageEvent::Disconnected(disconnected_instant))
            .await;
        assert_eq!(usage_handler.session_state, SessionState::NoSession);
    }

    #[tokio::test]
    async fn unsuccessfull_connection_test() {
        // Connection that is never established
        let mut usage_handler = mock_usage_handler();

        // Time info
        let start_day = time::OffsetDateTime::now_utc().date();
        let connect_request_instant = Instant::now();
        let connection_duration = Duration::from_millis(1234);

        let disconnect_request_instant = connect_request_instant + connection_duration;

        let disconnection_duration = Duration::from_millis(1234);
        let disconnected_instant = disconnect_request_instant + disconnection_duration;

        // Connection request
        usage_handler
            .handle_event(UsageEvent::ConnectRequest(connect_request_instant))
            .await;
        assert_eq!(
            usage_handler.session_state,
            SessionState::ConnectionRequested {
                start_day,
                start_time: connect_request_instant,
            }
        );

        // Connecting events
        usage_handler
            .handle_event(UsageEvent::Connecting {
                instant: Instant::now(),
                retry_attempt: 0,
                maybe_exit_id: None,
                maybe_exit_cc: None,
                maybe_tunnel_type: None,
            })
            .await;
        assert_eq!(
            usage_handler.session_state,
            SessionState::Connecting {
                start_day,
                start_time: connect_request_instant,
                retry_attempt: 0,
                tunnel_type: None,
                exit_id: None,
                exit_cc: None,
                follow_up_id: None,
            }
        );

        // Info update and skip a bit forward
        usage_handler
            .handle_event(UsageEvent::Connecting {
                instant: Instant::now(),
                retry_attempt: 4,
                maybe_exit_id: Some(mock_gateway_id_a()),
                maybe_exit_cc: Some(mock_gateway_cc_a()),
                maybe_tunnel_type: None,
            })
            .await;
        assert_eq!(
            usage_handler.session_state,
            SessionState::Connecting {
                start_day,
                start_time: connect_request_instant,
                retry_attempt: 4,
                exit_id: Some(mock_gateway_id_a()),
                exit_cc: Some(mock_gateway_cc_a()),
                tunnel_type: None,
                follow_up_id: None,
            }
        );

        // Disconnection request
        usage_handler
            .handle_event(UsageEvent::DisconnectRequest(disconnect_request_instant))
            .await;

        assert_eq!(
            usage_handler.session_state,
            SessionState::Disconnecting {
                start_day,
                connection_duration,
                retry_attempt: 4,
                session_duration: Duration::from_secs(0),
                tunnel_type: "".into(),
                exit_id: mock_gateway_id_a(),
                exit_cc: Some(mock_gateway_cc_a()),
                disconnecting_time: disconnect_request_instant,
                follow_up_id: None,
            }
        );

        // Disconnected and session end
        usage_handler
            .handle_event(UsageEvent::Disconnected(disconnected_instant))
            .await;
        assert_eq!(usage_handler.session_state, SessionState::NoSession);

        // Test that storage is correct
        let expected_report = SessionReport {
            day_utc: start_day,
            connection_time_ms: connection_duration.as_millis().try_into().unwrap(),
            retry_attempt: 4,
            session_duration_min: 0,
            tunnel_type: "".into(),
            exit_id: mock_gateway_id_a(),
            exit_cc: Some(mock_gateway_cc_a()),
            follow_up_id: None,
            error: None,
            disconnection_time_ms: disconnection_duration.as_millis().try_into().unwrap(),
        };

        usage_handler.assert_stored_session(expected_report);
    }

    #[test]
    fn bucketize_session_duration_test() {
        // Zero duration gets a zero
        assert_eq!(bucketize_session_duration(0), 0);
        // Sub 60 gets a 1
        assert_eq!(bucketize_session_duration(1), 1);
        assert_eq!(bucketize_session_duration(2), 1);
        assert_eq!(bucketize_session_duration(30), 1);
        assert_eq!(bucketize_session_duration(59), 1);
        // higher
        assert!(bucketize_session_duration(60) > 1);
    }
}
