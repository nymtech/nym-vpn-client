// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(not(target_os = "ios"))]
use futures::channel::{mpsc, oneshot};
#[cfg(not(target_os = "ios"))]
use nym_sdk::TaskClient;
#[cfg(not(target_os = "ios"))]
use std::sync::{Arc, Mutex};
#[cfg(not(target_os = "ios"))]
use talpid_routing::RouteManager;
#[cfg(not(target_os = "ios"))]
use talpid_tunnel::{tun_provider::TunProvider, TunnelEvent};

#[cfg(not(target_os = "ios"))]
use crate::{
    config::WireguardConfig,
    error::Result,
    tunnel::{start_tunnel, Tunnel},
    tunnel_setup::WgTunnelSetup,
    WireguardConnectionInfo,
};

#[cfg(not(target_os = "ios"))]
pub async fn create_wireguard_tunnel(
    route_manager: &RouteManager,
    shutdown: TaskClient,
    tun_provider: Arc<Mutex<TunProvider>>,
    wireguard_config: WireguardConfig,
) -> Result<(
    WgTunnelSetup,
    mpsc::UnboundedReceiver<(TunnelEvent, oneshot::Sender<()>)>,
)> {
    tracing::debug!("Creating wireguard tunnel");
    let handle = route_manager.handle()?;
    let tunnel = Tunnel::new(wireguard_config.clone(), handle, tun_provider);

    let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
    let (tunnel_handle, event_rx, tunnel_close_tx) =
        start_tunnel(&tunnel, shutdown, finished_shutdown_tx)?;

    let wireguard_waiting = WgTunnelSetup {
        connection_info: WireguardConnectionInfo {
            gateway_id: wireguard_config.gateway_id,
            public_key: wireguard_config
                .talpid_config
                .tunnel
                .private_key
                .public_key()
                .to_string(),
            private_ipv4: wireguard_config.gateway_data.private_ipv4,
        },
        receiver: finished_shutdown_rx,
        handle: tunnel_handle,
        tunnel_close_tx,
    };

    Ok((wireguard_waiting, event_rx))
}
