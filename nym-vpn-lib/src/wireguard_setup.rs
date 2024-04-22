// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::oneshot::{self, Receiver, Sender};
use log::*;
use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};
use talpid_routing::RouteManager;
use talpid_tunnel::tun_provider::TunProvider;
use tap::TapFallible;

use crate::{
    error::Result,
    init_wireguard_config,
    routing::{self, TunnelGatewayIp},
    tunnel::{setup_route_manager, start_tunnel, Tunnel},
    tunnel_setup::WgTunnelSetup,
    util::handle_interrupt,
    wg_gateway_client::WgGatewayClient,
};
use nym_gateway_directory::{GatewayClient, NodeIdentity};

pub struct WireguardSetup {
    pub route_manager: RouteManager,
    // The IP address of the gateway inside the tunnel. This will depend on if wireguard is
    // enabled
    pub tunnel_gateway_ip: TunnelGatewayIp,
    pub tunnel_close_tx: Sender<()>,
}

pub async fn empty_wireguard_setup() -> Result<(WireguardSetup, Receiver<()>)> {
    info!("Setting up route manager");
    let route_manager = setup_route_manager().await?;
    let tunnel_gateway_ip = routing::TunnelGatewayIp::new(None);
    let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();

    Ok((
        WireguardSetup {
            route_manager,
            tunnel_gateway_ip,
            tunnel_close_tx,
        },
        tunnel_close_rx,
    ))
}

pub async fn create_wireguard_tunnel(
    private_key: &str,
    wg_ip: Ipv4Addr,
    tun_provider: Arc<Mutex<TunProvider>>,
    gateway_client: &GatewayClient,
    wg_gateway_client: &WgGatewayClient,
    gateway_identity: &NodeIdentity,
) -> Result<(WireguardSetup, WgTunnelSetup, Tunnel)> {
    let (mut wireguard_setup, tunnel_close_rx) = empty_wireguard_setup().await?;
    let wireguard_config = init_wireguard_config(
        gateway_client,
        wg_gateway_client,
        &gateway_identity.to_base58_string(),
        private_key,
        wg_ip.into(),
    )
    .await?;

    wireguard_setup.tunnel_gateway_ip =
        routing::TunnelGatewayIp::new(Some(wireguard_config.clone()));

    info!("Creating tunnel");
    let tunnel = match Tunnel::new(
        wireguard_config.clone(),
        wireguard_setup.route_manager.handle()?,
        tun_provider,
    ) {
        Ok(tunnel) => tunnel,
        Err(err) => {
            error!("Failed to create tunnel: {err}");
            debug!("{err:?}");
            // Ignore if these fail since we're interesting in the original error anyway
            handle_interrupt(
                wireguard_setup.route_manager,
                None,
                wireguard_setup.tunnel_close_tx,
            )
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
        handle: tunnel_handle,
    };

    Ok((wireguard_setup, wireguard_waiting, tunnel))
}
