// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::oneshot;
use log::*;
use std::sync::{Arc, Mutex, RwLock};
use talpid_routing::RouteManager;
use talpid_tunnel::tun_provider::TunProvider;
use tap::TapFallible;

use crate::{
    config::WireguardConfig,
    error::{Error, Result},
    tunnel::{start_tunnel, Tunnel},
    tunnel_setup::WgTunnelSetup,
    util::handle_interrupt,
};

pub async fn create_wireguard_tunnel(
    route_manager: Arc<RwLock<RouteManager>>,
    tun_provider: Arc<Mutex<TunProvider>>,
    wireguard_config: WireguardConfig,
) -> Result<WgTunnelSetup> {
    let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();

    let handle = route_manager
        .read()
        .map_err(|_| Error::RouteManagerPoisonedLock)?
        .handle()?;
    let tunnel = match Tunnel::new(wireguard_config, handle, tun_provider) {
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

    let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
    let tunnel_handle = start_tunnel(&tunnel, tunnel_close_rx, finished_shutdown_tx)?;

    let wireguard_waiting = WgTunnelSetup {
        receiver: finished_shutdown_rx,
        tunnel_close_tx,
        handle: tunnel_handle,
    };

    Ok(wireguard_waiting)
}
