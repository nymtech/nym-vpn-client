// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Shows the tunnel as an active VPN in NetworkManager, whichever client connected it.
//!
//! Whenever the tunnel comes up, the NymVPN connection is activated in NetworkManager unless it is
//! active already. NetworkManager then starts the nm-nymvpn-service plugin, which follows the
//! tunnel from there. Nothing happens when NetworkManager is not running or has no NymVPN profile.

use anyhow::Context;
use nym_dbus::network_manager::NetworkManager;
use nym_vpn_lib_types::TunnelState;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

const SERVICE_TYPE: &str = "org.freedesktop.NetworkManager.nymvpn";

/// Activate the NymVPN connection each time the tunnel comes up, until `shutdown` is cancelled.
pub async fn run(mut tunnel_state: watch::Receiver<TunnelState>, shutdown: CancellationToken) {
    let mut was_connected = is_connected(&tunnel_state);
    loop {
        tokio::select! {
            changed = tunnel_state.changed() => {
                if changed.is_err() {
                    break;
                }
            }
            _ = shutdown.cancelled() => break,
        }
        let connected = is_connected(&tunnel_state);
        let came_up = connected && !was_connected;
        was_connected = connected;
        if !came_up {
            continue;
        }
        match with_network_manager(|nm| nm.activate_vpn_connection(SERVICE_TYPE)).await {
            Ok(Some(uuid)) => tracing::info!("Activated NetworkManager connection {uuid}"),
            Ok(None) => {}
            Err(err) => tracing::debug!("Cannot activate the NetworkManager connection: {err:#}"),
        }
    }
}

fn is_connected(tunnel_state: &watch::Receiver<TunnelState>) -> bool {
    matches!(*tunnel_state.borrow(), TunnelState::Connected { .. })
}

/// Run a blocking NetworkManager call off the async runtime.
async fn with_network_manager<T: Send + 'static>(
    call: impl FnOnce(&NetworkManager) -> nym_dbus::network_manager::Result<T> + Send + 'static,
) -> anyhow::Result<T> {
    let result = tokio::task::spawn_blocking(move || call(&NetworkManager::new()?))
        .await
        .context("NetworkManager task failed")?;
    Ok(result?)
}
