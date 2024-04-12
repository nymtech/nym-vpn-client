// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use futures::StreamExt;
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

    async fn handle_status_message(&self, msg: SentStatus) {
        debug!("Received status: {msg}");
        match msg.downcast_ref::<TaskStatus>() {
            Some(TaskStatus::Ready) => {
                info!("VPN status: connected");
                self.set_shared_state(VpnState::Connected);
                return;
            }
            Some(TaskStatus::ReadyWithGateway(_)) => {
                info!("VPN status: connected");
                self.set_shared_state(VpnState::Connected);
                return;
            }
            None => {
                info!("VPN status: unknown: {msg}");
            }
        }
        match msg.downcast_ref::<ConnectionMonitorStatus>() {
            Some(e) => info!("VPN status: {e}"),
            None => info!("VPN status: unknown: {msg}"),
        }
    }

    pub(super) async fn start(self, mut vpn_status_rx: StatusReceiver) {
        tokio::spawn(async move {
            while let Some(msg) = vpn_status_rx.next().await {
                self.handle_status_message(msg).await;
            }
        });
    }

    fn set_shared_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }
}
