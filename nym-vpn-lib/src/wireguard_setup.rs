// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::oneshot;
use log::*;
use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex, RwLock},
};
use talpid_routing::RouteManager;
use talpid_tunnel::tun_provider::TunProvider;
use tap::TapFallible;

use crate::{
    error::{Error, Result},
    init_wireguard_config,
    tunnel::{start_tunnel, Tunnel},
    tunnel_setup::WgTunnelSetup,
    util::handle_interrupt,
    wg_gateway_client::WgGatewayClient,
};
use nym_gateway_directory::{GatewayClient, NodeIdentity};

pub async fn create_wireguard_tunnel(
    private_key: &str,
    wg_ip: Ipv4Addr,
    route_manager: Arc<RwLock<RouteManager>>,
    tun_provider: Arc<Mutex<TunProvider>>,
    gateway_client: &GatewayClient,
    wg_gateway_client: &WgGatewayClient,
    gateway_identity: &NodeIdentity,
) -> Result<(WgTunnelSetup, Tunnel)> {
    let wireguard_config = init_wireguard_config(
        gateway_client,
        wg_gateway_client,
        &gateway_identity.to_base58_string(),
        private_key,
        wg_ip.into(),
    )
    .await?;
    let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();

    let handle = route_manager
        .read()
        .map_err(|_| Error::RouteManagerPoisonedLock)?
        .handle()?;
    info!("Creating tunnel");
    let tunnel = match Tunnel::new(wireguard_config.clone(), handle, tun_provider) {
        Ok(tunnel) => tunnel,
        Err(err) => {
            error!("Failed to create tunnel: {err}");
            debug!("{err:?}");
            // Ignore if these fail since we're interesting in the original error anyway
            handle_interrupt(route_manager, None)
                .await
                .tap_err(|err| {
                    warn!("Failed to handle interrupt: {err}");
                })
                .ok();
            return Err(err);
        }
    };

    info!("Starting wireguard tunnel");
    let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
    let tunnel_handle = start_tunnel(&tunnel, tunnel_close_rx, finished_shutdown_tx)?;

    let wireguard_waiting = WgTunnelSetup {
        receiver: finished_shutdown_rx,
        tunnel_close_tx,
        handle: tunnel_handle,
    };

    Ok((wireguard_waiting, tunnel))
}
