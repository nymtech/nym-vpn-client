// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::{mpsc, oneshot};
use nym_sdk::TaskClient;
use tracing::debug;
use std::sync::{Arc, Mutex};
use talpid_routing::RouteManager;
use talpid_tunnel::{tun_provider::TunProvider, TunnelEvent};

use crate::{
    config::WireguardConfig,
    error::Result,
    tunnel::{start_tunnel, Tunnel},
    tunnel_setup::WgTunnelSetup,
};

pub async fn create_wireguard_tunnel(
    route_manager: &RouteManager,
    shutdown: TaskClient,
    tun_provider: Arc<Mutex<TunProvider>>,
    wireguard_config: WireguardConfig,
) -> Result<(
    WgTunnelSetup,
    mpsc::UnboundedReceiver<(TunnelEvent, oneshot::Sender<()>)>,
)> {
    debug!("Creating wireguard tunnel");
    let handle = route_manager.handle()?;
    let tunnel = Tunnel::new(wireguard_config, handle, tun_provider);

    let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
    let (tunnel_handle, event_rx) = start_tunnel(&tunnel, shutdown, finished_shutdown_tx)?;

    let wireguard_waiting = WgTunnelSetup {
        receiver: finished_shutdown_rx,
        handle: tunnel_handle,
    };

    Ok((wireguard_waiting, event_rx))
}
