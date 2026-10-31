// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Shows the tunnel as an active VPN in NetworkManager, whichever client connected it.
//!
//! Whenever the tunnel comes up, the NymVPN connection is activated in NetworkManager unless it is
//! active already. NetworkManager then starts the nm-nymvpn-service plugin, which follows the
//! tunnel from there. Nothing happens when NetworkManager is not running or has no NymVPN profile.

use nym_common::ErrorExt;
use nym_dbus::network_manager::NetworkManager;
use nym_vpn_lib_types::TunnelState;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

const SERVICE_TYPE: &str = "org.freedesktop.NetworkManager.nymvpn";

/// Activate the NymVPN connection each time the tunnel comes up, until `shutdown` is cancelled.
pub async fn run(mut tunnel_state_rx: watch::Receiver<TunnelState>, shutdown: CancellationToken) {
    let mut is_active = false;

    loop {
        tokio::select! {
            changed = tunnel_state_rx.changed() => {
                if changed.is_err() {
                    break;
                }
            }
            _ = shutdown.cancelled() => break,
        }

        let tunnel_state = tunnel_state_rx.borrow().clone();
        match tunnel_state {
            TunnelState::Connected { .. } | TunnelState::Connecting { .. } => {
                if !is_active {
                    is_active = activate_connection().await;
                }
            }
            TunnelState::Disconnecting { .. } => {}
            TunnelState::Disconnected | TunnelState::Offline { .. } | TunnelState::Error(..) => {
                if is_active {
                    is_active = deactivate_connection().await;
                }
            }
        }
    }
}

async fn activate_connection() -> bool {
    let result = tokio::task::spawn_blocking(|| {
        match NetworkManager::new().and_then(|nm| nm.activate_vpn_connection(SERVICE_TYPE)) {
            Ok(uuid) => {
                if let Some(uuid) = uuid {
                    tracing::info!("Activated NetworkManager connection {uuid}");
                    true
                } else {
                    false
                }
            }
            Err(err) => {
                tracing::debug!(
                    "{}",
                    err.display_chain_with_msg("Cannot activate the NetworkManager connection")
                );
                false
            }
        }
    })
    .await;

    if let Err(err) = result.as_ref() {
        tracing::debug!("NetworkManager task failed: {err}");
    }

    result.unwrap_or_default()
}

async fn deactivate_connection() -> bool {
    let result = tokio::task::spawn_blocking(|| {
        match NetworkManager::new().and_then(|nm| nm.deactivate_vpn_connection(SERVICE_TYPE)) {
            Ok(uuid) => {
                if let Some(uuid) = uuid {
                    tracing::info!("Deactivated NetworkManager connection {uuid}");
                    true
                } else {
                    false
                }
            }
            Err(err) => {
                tracing::debug!(
                    "{}",
                    err.display_chain_with_msg("Cannot activate the NetworkManager connection")
                );
                false
            }
        }
    })
    .await;

    if let Err(err) = result.as_ref() {
        tracing::debug!("NetworkManager task failed: {err}");
    }

    result.unwrap_or_default()
}
