// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use nym_task::StatusSender;
use nym_vpn_lib::{
    connection_monitor::ConnectionMonitorStatus, SentStatus, StatusReceiver, TaskStatus,
};
use tracing::{debug, info};

use super::vpn_service::VpnState;

pub(super) struct VpnServiceStatusListener {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
}

impl VpnServiceStatusListener {
    pub(super) fn new(shared_vpn_state: Arc<std::sync::Mutex<VpnState>>) -> Self {
        Self { shared_vpn_state }
    }

    async fn handle_status_message(&self, msg: SentStatus) -> SentStatus {
        debug!("Received status: {msg}");
        if let Some(msg) = msg.downcast_ref::<TaskStatus>() {
            match msg {
                TaskStatus::Ready => {
                    info!("VPN status: connected");
                    self.set_shared_state(VpnState::Connected);
                }
                TaskStatus::ReadyWithGateway(gateway) => {
                    info!("VPN status: connected to gateway: {gateway}");
                    self.set_shared_state(VpnState::Connected);
                }
            }
        } else if let Some(msg) = msg.downcast_ref::<ConnectionMonitorStatus>() {
            info!("VPN status: {msg}");
            // match msg {
            //     ConnectionMonitorStatus::ConnectedIpv4 | ConnectionMonitorStatus::ConnectedIpv6 => {
            //     }
            //     msg => {
            //         info!("VPN status: {msg}");
            //     }
            // }
        } else {
            info!("VPN status: unknown: {msg}");
        }
        msg
    }

    pub(super) async fn start(
        self,
        mut vpn_status_rx: StatusReceiver,
        mut listener_vpn_status_tx: StatusSender,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = vpn_status_rx.next().await {
                let listener_msg = self.handle_status_message(msg).await;

                // Forward the status message to the command listener
                listener_vpn_status_tx.send(listener_msg).await.unwrap();
            }
        });
    }

    fn set_shared_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }
}
