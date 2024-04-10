// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use futures::StreamExt;
use tracing::{debug, info};

use super::vpn_service::VpnState;

pub(super) struct VpnServiceStatusListener {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
}

impl VpnServiceStatusListener {
    pub(super) fn new(shared_vpn_state: Arc<std::sync::Mutex<VpnState>>) -> Self {
        Self { shared_vpn_state }
    }

    pub(super) async fn start(
        self,
        mut vpn_status_rx: futures::channel::mpsc::Receiver<
            Box<dyn std::error::Error + Send + Sync>,
        >,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = vpn_status_rx.next().await {
                debug!("Received status: {msg}");
                match msg.downcast_ref::<nym_vpn_lib::TaskStatus>() {
                    Some(nym_vpn_lib::TaskStatus::Ready) => {
                        info!("VPN status: connected");
                        self.set_shared_state(VpnState::Connected);
                        continue;
                    }
                    Some(nym_vpn_lib::TaskStatus::ReadyWithGateway(_)) => {
                        info!("VPN status: connected");
                        self.set_shared_state(VpnState::Connected);
                        continue;
                    }
                    None => {}
                }
                match msg.downcast_ref::<nym_vpn_lib::connection_monitor::ConnectionMonitorStatus>()
                {
                    Some(e) => {
                        info!("VPN status: {e}");
                        continue;
                    }
                    None => info!("VPN status: unknown: {msg}"),
                }
            }
        });
    }

    fn set_shared_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }
}
