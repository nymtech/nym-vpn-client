// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use futures::{channel::mpsc, StreamExt};
use nym_task::TaskClient;
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::error::Result;

const CONNECTION_MONITOR_REPORT_INTERVAL: Duration = Duration::from_secs(5);

// When the latest successful self ping is older than this, we consider the connection to the
// entry gateway as down
const SELF_PING_EXPIRY: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub enum ConnectionStatusEvent {
    MixnetSelfPing,
}

#[derive(Debug, Default)]
struct ConnectionStats {
    // TODO: extend with all sorts of good stuff
    latest_self_ping: Option<Instant>,
}

struct ConnectionMonitor {
    connection_event_rx: mpsc::UnboundedReceiver<ConnectionStatusEvent>,
    stats: ConnectionStats,
}

impl ConnectionMonitor {
    fn new(connection_event_rx: mpsc::UnboundedReceiver<ConnectionStatusEvent>) -> Self {
        ConnectionMonitor {
            connection_event_rx,
            stats: ConnectionStats::default(),
        }
    }

    async fn run(mut self, mut task_client: TaskClient) -> Result<()> {
        debug!("Connection monitor is running");
        let mut report_interval = tokio::time::interval(CONNECTION_MONITOR_REPORT_INTERVAL);
        // Reset so that we don't send a report immediately before we even have a change for any
        // self pings to be sent and received
        report_interval.reset();

        loop {
            tokio::select! {
                _ = task_client.recv() => {
                    trace!("ConnectionMonitor: Received shutdown");
                    break;
                }
                Some(event) = self.connection_event_rx.next() => {
                    match event {
                        ConnectionStatusEvent::MixnetSelfPing => {
                            trace!("Received self ping event");
                            self.stats.latest_self_ping = Some(Instant::now());
                        }
                    }
                }
                _ = report_interval.tick() => {
                    debug!("Connection statistics: {:?}", self.stats);

                    // Send I'm alive messages, so listerners can hear that we are still there
                    task_client.send_status_msg(Box::new(ConnectionMonitorStatus::ImAlive));

                    if let Some(latest_self_ping) = self.stats.latest_self_ping {
                        if latest_self_ping.elapsed() > SELF_PING_EXPIRY {
                            error!("Entry gateway not routing our mixnet traffic");
                            task_client.send_status_msg(Box::new(ConnectionMonitorStatus::EntryGatewayDown));
                        }
                    } else {
                        error!("Entry gateway not routing our mixnet traffic");
                        task_client.send_status_msg(Box::new(ConnectionMonitorStatus::EntryGatewayDown));
                    }
                }
            }
        }
        debug!("ConnectionMonitor: Exiting");
        Ok(())
    }
}

// Just like in nym_task::TaskManager and TaskStatus, strictly speaking this is not an error, but a
// status message. We're just piggybacking on the error trait for now. In the future, we might want
// to create a separate trait in nym_task::TaskManager
#[derive(thiserror::Error, Debug)]
pub enum ConnectionMonitorStatus {
    #[error("I'm alive")]
    ImAlive,
    #[error("entry gateway appears down - it's not routing our mixnet traffic")]
    EntryGatewayDown,
}

pub fn start_connection_monitor(
    connection_event_rx: futures::channel::mpsc::UnboundedReceiver<ConnectionStatusEvent>,
    shutdown_listener: TaskClient,
) -> JoinHandle<Result<()>> {
    debug!("Creating connection monitor");
    let monitor = ConnectionMonitor::new(connection_event_rx);
    tokio::spawn(async move {
        monitor.run(shutdown_listener).await.inspect_err(|err| {
            error!("Connection monitor error: {err}");
        })
    })
}
