// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    platform::{uniffi_set_listener_status, RUNTIME},
    uniffi_custom_impls::{StatusEvent, TunStatus},
    NymVpnStatusMessage,
};
use nym_bandwidth_controller_pre_ecash::BandwidthStatusMessage;
use nym_connection_monitor::ConnectionMonitorStatus;
use nym_task::{
    manager::{SentStatus, TaskStatus},
    StatusReceiver,
};
use tokio_stream::StreamExt;
use tracing::debug;

pub(super) struct VpnServiceStatusListener {}

impl VpnServiceStatusListener {
    pub(super) fn new() -> Self {
        Self {}
    }

    async fn handle_status_message(&self, status_update: SentStatus) -> SentStatus {
        debug!("Received status: {status_update}");

        if let Some(message) = status_update.downcast_ref::<TaskStatus>() {
            match message {
                TaskStatus::Ready => debug!("Started Nym VPN"),
                TaskStatus::ReadyWithGateway(gateway) => {
                    debug!("Started Nym VPN: connected to {gateway}");
                    uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Up));
                }
            }
        }

        if let Some(message) = status_update.downcast_ref::<BandwidthStatusMessage>() {
            uniffi_set_listener_status(StatusEvent::Bandwidth(message.into()))
        }

        if let Some(message) = status_update
            .downcast_ref::<ConnectionMonitorStatus>()
            .cloned()
        {
            uniffi_set_listener_status(StatusEvent::Connection(message.into()))
        }

        if let Some(message) = status_update.downcast_ref::<NymVpnStatusMessage>().cloned() {
            uniffi_set_listener_status(StatusEvent::NymVpn(message.into()))
        }
        status_update
    }

    pub(super) async fn start(self, mut vpn_status_rx: StatusReceiver) {
        RUNTIME.spawn(async move {
            while let Some(message) = vpn_status_rx.next().await {
                self.handle_status_message(message).await;
            }
        });
    }
}
