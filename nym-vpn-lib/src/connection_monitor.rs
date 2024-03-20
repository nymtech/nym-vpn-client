// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use futures::StreamExt;
use nym_task::TaskClient;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::error::Result;

#[derive(Debug)]
pub enum ConnectionEvent {
    MixnetSelfPing,
}

#[derive(Debug, Default)]
struct ConnectionStats {
    latest_self_ping: Option<Instant>,
}

struct ConnectionMonitor {
    connection_event_rx: futures::channel::mpsc::UnboundedReceiver<ConnectionEvent>,
    stats: ConnectionStats,
}

impl ConnectionMonitor {
    fn new(
        connection_event_rx: futures::channel::mpsc::UnboundedReceiver<ConnectionEvent>,
    ) -> Self {
        ConnectionMonitor {
            connection_event_rx,
            stats: ConnectionStats::default(),
        }
    }

    async fn run(mut self, mut task_client: TaskClient) -> Result<()> {
        info!("Connection monitor is running");
        loop {
            tokio::select! {
                _ = task_client.recv_with_delay() => {
                    info!("ConnectionMonitor: Received shutdown");
                    break;
                }
                Some(event) = self.connection_event_rx.next() => {
                    match event {
                        ConnectionEvent::MixnetSelfPing => {
                            info!("Received self ping event");
                            self.stats.latest_self_ping = Some(Instant::now());
                        }
                    }
                }
                // Every 5 seconds, check if we have received a self-ping in the last 5 seconds
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    let msg = ConnectionMonitorStatus::Hello;
                    task_client.send_status_msg(Box::new(msg));
                    if let Some(latest_self_ping) = self.stats.latest_self_ping {
                        if latest_self_ping.elapsed() > Duration::from_secs(5) {
                            error!("Haven't received a self-ping in the last 5 seconds");
                            error!("Connection to entry gateway seems down!");
                            // WIP(JON): we need to make sure things are not put in a pile with no
                            // receiver emptying it
                            let msg = ConnectionMonitorStatus::EntryGatewayDown;
                            task_client.send_status_msg(Box::new(msg));
                        }
                    } else {
                        error!("Haven't received a self-ping yet");
                        error!("Connection to entry gateway seems down!");
                        let msg = ConnectionMonitorStatus::EntryGatewayDown;
                        task_client.send_status_msg(Box::new(msg));
                    }
                }
            }
        }
        info!("ConnectionMonitor: Exiting");
        Ok(())
    }
}

// Just like in nym_task::TaskManager and TaskStatus, strictly speaking this is not an error, but a
// status message. We're just piggybacking on the error trait for now. In the future, we might want
// to create a separate trait in nym_task::TaskManager
#[derive(thiserror::Error, Debug)]
pub enum ConnectionMonitorStatus {
    #[error("hello")]
    Hello,
    #[error("connection to entry gateway seems down!")]
    EntryGatewayDown,
}

pub fn start_connection_monitor(
    connection_event_rx: futures::channel::mpsc::UnboundedReceiver<ConnectionEvent>,
    shutdown_listener: TaskClient,
) -> JoinHandle<Result<()>> {
    info!("Creating connection monitor");
    let monitor = ConnectionMonitor::new(connection_event_rx);
    tokio::spawn(async move {
        let ret = monitor.run(shutdown_listener).await;
        if let Err(err) = ret {
            error!("Connection monitor error: {err}");
            Err(err)
        } else {
            ret
        }
    })
}
