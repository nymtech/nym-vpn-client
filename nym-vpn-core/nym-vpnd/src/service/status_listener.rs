// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{SinkExt, StreamExt};
use nym_task::StatusSender;
use nym_vpn_lib::{
    connection_monitor::ConnectionMonitorStatus, NymVpnStatusMessage2, SentStatus, StatusReceiver,
    TaskStatus,
};
use time::OffsetDateTime;
use tracing::{debug, info};

use super::vpn_service::{SharedVpnState, VpnState};

pub(super) struct VpnServiceStatusListener {
    shared_vpn_state: SharedVpnState,
}

impl VpnServiceStatusListener {
    pub(super) fn new(shared_vpn_state: SharedVpnState) -> Self {
        Self { shared_vpn_state }
    }

    async fn handle_status_message(&self, msg: SentStatus) -> SentStatus {
        debug!("Received status: {msg}");
        if let Some(msg) = msg.downcast_ref::<TaskStatus>() {
            // match msg {
            //     TaskStatus::Ready => {
            //         info!("VPN status: connected");
            //         self.shared_vpn_state.set(VpnState::Connected {
            //             gateway: "unknown".to_string(),
            //             since: OffsetDateTime::now_utc(),
            //         });
            //     }
            //     TaskStatus::ReadyWithGateway(gateway) => {
            //         info!("VPN status: connected to gateway: {gateway}");
            //         self.shared_vpn_state.set(VpnState::Connected {
            //             gateway: gateway.clone(),
            //             since: OffsetDateTime::now_utc(),
            //         });
            //     }
            // }
            info!("IGNORED VPN task status: {msg}");
        } else if let Some(msg) = msg.downcast_ref::<NymVpnStatusMessage2>() {
            info!("VPN status: {msg}");
            match msg {
                NymVpnStatusMessage2::MixnetConnectionInfo {
                    mixnet_connection_info,
                    mixnet_exit_connection_info,
                } => {
                    info!("VPN status: connected");
                    let nym_address = mixnet_connection_info.nym_address.clone();
                    let entry_gateway = mixnet_connection_info.entry_gateway.clone();
                    let exit_gateway = mixnet_exit_connection_info.exit_gateway.clone();
                    let exit_ipr = mixnet_exit_connection_info.exit_ipr.clone();
                    let ipv4 = mixnet_exit_connection_info.ips.ipv4;
                    let ipv6 = mixnet_exit_connection_info.ips.ipv6;
                    self.shared_vpn_state.set(VpnState::Connected {
                        nym_address,
                        entry_gateway,
                        exit_gateway,
                        exit_ipr,
                        ipv4,
                        ipv6,
                        since: OffsetDateTime::now_utc(),
                    });
                }
            }
        } else if let Some(msg) = msg.downcast_ref::<ConnectionMonitorStatus>() {
            info!("VPN connection monitor status: {msg}");
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

                // Forward the status message to the command listener so that it can provide these
                // on its streaming endpoints
                listener_vpn_status_tx.send(listener_msg).await.ok();
            }
        });
    }
}
