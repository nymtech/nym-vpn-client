// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{SinkExt, StreamExt};
use nym_bandwidth_controller::BandwidthStatusMessage;
use nym_task::StatusSender;
use nym_vpn_lib::{
    connection_monitor::ConnectionMonitorStatus, NymVpnStatusMessage, SentStatus, StatusReceiver,
    TaskStatus,
};
use time::OffsetDateTime;
use tracing::{debug, info};

use crate::service::vpn_service::VpnConnectedStateDetails;

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
            // For the vpn client we ignore the TaskStatus message that is sent when the connection
            // is established. We rely on the VPN status messages instead since they provide more
            // information.
            info!("Received ignored TaskStatus message: {msg}");
        } else if let Some(msg) = msg.downcast_ref::<NymVpnStatusMessage>() {
            info!("VPN status: {msg}");
            match msg {
                NymVpnStatusMessage::MixnetConnectionInfo {
                    mixnet_connection_info,
                    mixnet_exit_connection_info,
                } => {
                    let connected_details = VpnConnectedStateDetails {
                        nym_address: mixnet_connection_info.nym_address,
                        entry_gateway: mixnet_connection_info.entry_gateway,
                        exit_gateway: mixnet_exit_connection_info.exit_gateway,
                        exit_ipr: mixnet_exit_connection_info.exit_ipr,
                        ipv4: mixnet_exit_connection_info.ips.ipv4,
                        ipv6: mixnet_exit_connection_info.ips.ipv6,
                        since: OffsetDateTime::now_utc(),
                    };
                    self.shared_vpn_state
                        .set(VpnState::Connected(Box::new(connected_details)));
                }
            }
        } else if let Some(msg) = msg.downcast_ref::<ConnectionMonitorStatus>() {
            info!("VPN connection monitor status: {msg}");
        } else if let Some(msg) = msg.downcast_ref::<BandwidthStatusMessage>() {
            info!("VPN bandwidth status: monitor status: {msg}");
            match msg {
                BandwidthStatusMessage::RemainingBandwidth(_) => {}
                BandwidthStatusMessage::NoBandwidth => {}
            }
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
