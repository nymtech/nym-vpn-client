// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::StreamExt;
use nym_vpn_lib::connection_monitor::ConnectionMonitorStatus;
use nym_vpn_proto::{ConnectionStatus, ConnectionStatusUpdate};
use tracing::{debug, error, info};

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
                if let Err(err) = self.status_tx.send(ConnectionStatusUpdate {
                    message: message.to_string(),
                }) {
                    error!("Failed to broadcast connection status update: {:?}", err);
                }
            }
            nym_vpn_lib::TaskStatus::ReadyWithGateway(ref gateway) => {
                info!(
                    "Broadcasting connection status update ({gateway}): {:?}",
                    ConnectionStatus::Connected as i32
                );
                self.send_status_update(ConnectionStatusUpdate {
                    message: message.to_string(),
                });
            }
        }
    }

    fn handle_connection_monitor_status(&self, message: &ConnectionMonitorStatus) {
        // TODO: match on the message and send appropriate status
        self.send_status_update(ConnectionStatusUpdate {
            message: message.to_string(),
        });
    }

    fn send_status_update(&self, message: ConnectionStatusUpdate) {
        if let Err(err) = self.status_tx.send(message) {
            error!("Failed to broadcast connection status update: {:?}", err);
        }
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
                self.send_status_update(ConnectionStatusUpdate {
                    message: status_update.to_string(),
                });
            }
        }
        debug!("Status listener: exiting");
    }

    pub(super) fn start(self) {
        tokio::spawn(self.run());
    }
}
