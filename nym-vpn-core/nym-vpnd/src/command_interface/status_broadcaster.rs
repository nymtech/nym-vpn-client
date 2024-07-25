// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::StreamExt;
use nym_bandwidth_controller::BandwidthStatusMessage;
use nym_vpn_lib::{connection_monitor::ConnectionMonitorStatus, NymVpnStatusMessage};
use nym_vpn_proto::{connection_status_update::StatusType, ConnectionStatusUpdate};
use tracing::debug;

use super::protobuf::status_update::{
    status_update_from_bandwidth_status_message, status_update_from_monitor_status,
    status_update_from_status_message,
};

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
        // We ignore these messages for now, and rely on the vpn status messages instead since they
        // provide more information.
        debug!("Received ignored task status update: {:?}", message);
    }

    fn handle_status_message(&self, message: &NymVpnStatusMessage) {
        self.status_tx
            .send(status_update_from_status_message(message))
            .ok();
    }

    fn handle_connection_monitor_status(&self, message: &ConnectionMonitorStatus) {
        self.status_tx
            .send(status_update_from_monitor_status(message))
            .ok();
    }

    fn handle_bandwidth_status_message(&self, message: &BandwidthStatusMessage) {
        self.status_tx
            .send(status_update_from_bandwidth_status_message(message))
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
            } else if let Some(message) = status_update.downcast_ref::<NymVpnStatusMessage>() {
                self.handle_status_message(message);
            } else if let Some(message) = status_update.downcast_ref::<ConnectionMonitorStatus>() {
                self.handle_connection_monitor_status(message);
            } else if let Some(message) = status_update.downcast_ref::<BandwidthStatusMessage>() {
                self.handle_bandwidth_status_message(message);
            } else {
                self.status_tx
                    .send(ConnectionStatusUpdate {
                        kind: StatusType::Unknown as i32,
                        message: status_update.to_string(),
                        details: Default::default(),
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
