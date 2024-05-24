// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::StreamExt;
use nym_vpn_lib::connection_monitor::ConnectionMonitorStatus;
use nym_vpn_proto::{ConnectionStatus, ConnectionStatusUpdate};
use tracing::{debug, info};

pub(super) struct ConnectionStatusBroadcaster {
    status_tx: tokio::sync::broadcast::Sender<ConnectionStatusUpdate>,
    listener_vpn_status_rx: nym_task::StatusReceiver,
}

impl ConnectionStatusBroadcaster {
    pub(super) fn new(
        status_tx: tokio::sync::broadcast::Sender<ConnectionStatusUpdate>,
        listener_vpn_status_rx: nym_task::StatusReceiver,
    ) -> Self {
        Self {
            status_tx,
            listener_vpn_status_rx,
        }
    }

    fn handle_task_status(&self, message: &nym_vpn_lib::TaskStatus) {
        match message {
            nym_vpn_lib::TaskStatus::Ready => {
                info!(
                    "Broadcasting connection status update: {:?}",
                    ConnectionStatus::Connected as i32
                );
                self.status_tx
                    .send(ConnectionStatusUpdate {
                        message: message.to_string(),
                    })
                    .ok();
            }
            nym_vpn_lib::TaskStatus::ReadyWithGateway(ref gateway) => {
                info!(
                    "Broadcasting connection status update ({gateway}): {:?}",
                    ConnectionStatus::Connected as i32
                );
                self.status_tx
                    .send(ConnectionStatusUpdate {
                        message: message.to_string(),
                    })
                    .ok();
            }
        }
    }

    fn handle_connection_monitor_status(&self, message: &ConnectionMonitorStatus) {
        // TODO: match on the message and send appropriate status
        self.status_tx
            .send(ConnectionStatusUpdate {
                message: message.to_string(),
            })
            .ok();
    }

    async fn run(mut self) {
        while let Some(status_update) = self.listener_vpn_status_rx.next().await {
            debug!(
                "Received status update that we should broadcast: {:?}",
                status_update
            );
            if let Some(message) = status_update.downcast_ref::<nym_vpn_lib::TaskStatus>() {
                self.handle_task_status(message);
            } else if let Some(message) = status_update.downcast_ref::<ConnectionMonitorStatus>() {
                self.handle_connection_monitor_status(message);
            } else {
                self.status_tx
                    .send(ConnectionStatusUpdate {
                        message: status_update.to_string(),
                    })
                    .ok();
            }
        }
        debug!("Status listener: exiting");
    }

    pub(super) fn start(self) {
        tokio::spawn(self.run());
    }
}
