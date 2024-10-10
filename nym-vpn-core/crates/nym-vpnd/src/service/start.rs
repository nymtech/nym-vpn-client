// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use super::{vpn_service::NymVpnService, VpnServiceCommand, VpnServiceStateChange};

pub(crate) fn start_vpn_service(
    vpn_state_changes_tx: tokio::sync::broadcast::Sender<VpnServiceStateChange>,
    vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,
    shutdown_token: CancellationToken,
) -> JoinHandle<()> {
    tracing::info!("Starting VPN service");
    tokio::spawn(async {
        let service = NymVpnService::new(vpn_state_changes_tx, vpn_command_rx, shutdown_token);
        match service.init_storage().await {
            Ok(()) => {
                tracing::info!("VPN service initialized successfully");

                match service.run().await {
                    Ok(_) => {
                        tracing::info!("VPN service has successfully exited");
                    }
                    Err(e) => {
                        tracing::error!("VPN service has exited with error: {:?}", e);
                    }
                }
            }
            Err(err) => {
                tracing::error!("Failed to initialize VPN service: {:?}", err);
            }
        }
    })
}
