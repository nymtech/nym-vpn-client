// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::oneshot::{Receiver, Sender};
use futures::channel::{mpsc, oneshot};
use log::*;
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};
use talpid_routing::{RouteManager, RouteManagerHandle};
use talpid_tunnel::tun_provider::TunProvider;
use talpid_tunnel::TunnelArgs;
use talpid_wireguard::{config::Config, WireguardMonitor};
use tokio::task::JoinHandle;

use crate::config::WireguardConfig;

pub struct Tunnel {
    pub config: Config,
    pub route_manager_handle: RouteManagerHandle,
    pub tun_provider: Arc<Mutex<TunProvider>>,
}

impl Tunnel {
    pub fn new(
        config: WireguardConfig,
        route_manager_handle: RouteManagerHandle,
        tun_provider: Arc<Mutex<TunProvider>>,
    ) -> Self {
        Tunnel {
            config: config.0,
            route_manager_handle,
            tun_provider,
        }
    }
}

pub fn start_tunnel(
    tunnel: &Tunnel,
    tunnel_close_rx: Receiver<()>,
    finished_shutdown_tx: Sender<()>,
) -> Result<JoinHandle<Result<(), crate::error::Error>>, crate::error::Error> {
    let route_manager = tunnel.route_manager_handle.clone();
    // We only start the tunnel when we have wireguard enabled, and then we have the config
    let config = tunnel.config.clone();
    let id: Option<String> = config.tunnel.addresses.first().map(|a| a.to_string());
    let tun_provider = Arc::clone(&tunnel.tun_provider);
    let handle = tokio::task::spawn_blocking(move || -> Result<(), crate::error::Error> {
        let (event_tx, _) = mpsc::unbounded();
        let on_tunnel_event =
            move |event| -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
                let (tx, rx) = oneshot::channel::<()>();
                let _ = event_tx.unbounded_send((event, tx));
                Box::pin(async move {
                    let _ = rx.await;
                })
            };
        let mut resource_dir = std::env::temp_dir().join("nym-wg");
        if let Some(id) = id {
            resource_dir = resource_dir.join(id);
        }
        std::fs::create_dir_all(&resource_dir).unwrap();
        debug!("Tunnel resource dir: {:?}", resource_dir);
        let args = TunnelArgs {
            runtime: tokio::runtime::Handle::current(),
            resource_dir: &resource_dir,
            on_event: on_tunnel_event,
            tunnel_close_rx,
            tun_provider,
            retry_attempt: 3,
            route_manager,
        };
        let monitor = WireguardMonitor::start(
            config,
            None,
            Some(Path::new(&resource_dir.join("logs"))),
            args,
        )?;
        debug!("Wireguard monitor started, blocking current thread until shutdown");
        if let Err(e) = monitor.wait() {
            error!("Tunnel disconnected with error {:?}", e);
        } else {
            finished_shutdown_tx
                .send(())
                .map_err(|_| crate::error::Error::FailedToSendWireguardShutdown)?;
            debug!("Sent shutdown message");
        }
        Ok(())
    });

    Ok(handle)
}

pub async fn setup_route_manager() -> crate::error::Result<RouteManager> {
    #[cfg(target_os = "linux")]
    let route_manager = {
        let fwmark = 0;
        let table_id = 0;
        RouteManager::new(HashSet::new(), fwmark, table_id).await?
    };

    #[cfg(not(target_os = "linux"))]
    let route_manager = RouteManager::new(HashSet::new()).await?;

    Ok(route_manager)
}
