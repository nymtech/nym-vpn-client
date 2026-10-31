// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Activates the NymVPN connection in NetworkManager when a client other than NetworkManager asks
//! the daemon to connect, so the desktop shows the VPN as active.
//!
//! NetworkManager then starts the nm-nymvpn-service plugin, which follows the tunnel from there.
//! Nothing happens when NetworkManager is not running or has no NymVPN profile.

use std::time::Duration;

use anyhow::Context;
use nym_dbus::network_manager::NetworkManager;
use nym_vpn_lib_types::{TunnelEvent, TunnelState};
use tokio::sync::broadcast::{self, error::RecvError};

const SERVICE_TYPE: &str = "org.freedesktop.NetworkManager.nymvpn";

/// How long to wait after the tunnel came up before activating, so a NetworkManager connection
/// the plugin is bringing back up on its own is already visible.
const SETTLE: Duration = Duration::from_secs(2);

/// Activate the NymVPN connection in the background, unless NetworkManager already has it.
///
/// NetworkManager runs a VPN on top of its primary connection, so while offline this waits for
/// the tunnel to come up, and gives up if the daemon goes down first.
pub fn activate_vpn_connection(events: broadcast::Receiver<TunnelEvent>) {
    tokio::spawn(async move {
        match activate(events).await {
            Ok(Some(uuid)) => tracing::info!("Activated NetworkManager connection {uuid}"),
            Ok(None) => {}
            Err(err) => {
                tracing::debug!("Cannot activate the NetworkManager connection: {err:#}")
            }
        }
    });
}

async fn activate(mut events: broadcast::Receiver<TunnelEvent>) -> anyhow::Result<Option<String>> {
    if !with_network_manager(|nm| nm.has_primary_connection()).await? {
        tracing::debug!("Offline, activating the NetworkManager connection once the tunnel is up");
        if !wait_for_connected(&mut events).await {
            return Ok(None);
        }
        tokio::time::sleep(SETTLE).await;
    }
    with_network_manager(|nm| nm.activate_vpn_connection(SERVICE_TYPE)).await
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

/// Wait until the tunnel is up. Returns `false` if the daemon goes down instead.
async fn wait_for_connected(events: &mut broadcast::Receiver<TunnelEvent>) -> bool {
    loop {
        match events.recv().await {
            Ok(TunnelEvent::NewState(TunnelState::Connected { .. })) => return true,
            Ok(TunnelEvent::NewState(
                TunnelState::Disconnected
                | TunnelState::Error(_)
                | TunnelState::Offline { reconnect: false },
            ))
            | Err(RecvError::Closed) => return false,
            Ok(_) | Err(RecvError::Lagged(_)) => {}
        }
    }
}
