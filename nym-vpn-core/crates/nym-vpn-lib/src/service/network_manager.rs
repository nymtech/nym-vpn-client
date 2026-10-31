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
pub async fn run(mut tunnel_state: watch::Receiver<TunnelState>, shutdown: CancellationToken) {
    let mut was_connected = tunnel_state.borrow().is_connected();
    loop {
        tokio::select! {
            changed = tunnel_state.changed() => {
                if changed.is_err() {
                    break;
                }
            }
            _ = shutdown.cancelled() => break,
        }

        let connected = tunnel_state.borrow().is_connected();
        let came_up = connected && !was_connected;
        was_connected = connected;
        if !came_up {
            continue;
        }

        let result = tokio::task::spawn_blocking(|| {
            match NetworkManager::new().and_then(|nm| nm.activate_vpn_connection(SERVICE_TYPE)) {
                Ok(uuid) => {
                    if let Some(uuid) = uuid {
                        tracing::info!("Activated NetworkManager connection {uuid}");
                    }
                }
                Err(err) => {
                    tracing::debug!(
                        "{}",
                        err.display_chain_with_msg("Cannot activate the NetworkManager connection")
                    );
                }
            }
        })
        .await;

        if let Err(err) = result {
            tracing::debug!("NetworkManager task failed: {err}");
        }
    }
}
