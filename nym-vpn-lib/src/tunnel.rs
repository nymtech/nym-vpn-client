// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::oneshot::{Receiver, Sender};
use futures::channel::{mpsc, oneshot};
use log::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use talpid_core::dns::DnsMonitor;
use talpid_core::firewall::Firewall;
use talpid_routing::{RouteManager, RouteManagerHandle};
use talpid_tunnel::tun_provider::TunProvider;
use talpid_tunnel::TunnelArgs;
use talpid_wireguard::{config::Config, WireguardMonitor};
use tokio::task::JoinHandle;

use crate::config::WireguardConfig;

pub struct Tunnel {
    pub config: Option<Config>,
    pub firewall: Firewall,
    pub dns_monitor: DnsMonitor,
    pub route_manager_handle: RouteManagerHandle,
}

impl Tunnel {
    pub fn new(
        config: Option<WireguardConfig>,
        route_manager_handle: RouteManagerHandle,
    ) -> Result<Self, crate::error::Error> {
        #[cfg(not(target_os = "linux"))]
        let (firewall, dns_monitor) = {
            let (command_tx, _) = mpsc::unbounded();
            let command_tx = Arc::new(command_tx);
            let weak_command_tx = Arc::downgrade(&command_tx);
            debug!("Starting firewall");
            let firewall = Firewall::new()?;
            debug!("Starting dns monitor");
            let dns_monitor = DnsMonitor::new(weak_command_tx)?;
            (firewall, dns_monitor)
        };

        #[cfg(target_os = "linux")]
        let (firewall, dns_monitor) = {
            let fwmark = 0; // ?
            debug!("Starting firewall");
            let firewall = Firewall::new(fwmark)?;
            debug!("Starting dns monitor");
            let dns_monitor = DnsMonitor::new(
                tokio::runtime::Handle::current(),
                route_manager_handle.clone(),
            )?;
            (firewall, dns_monitor)
        };

        Ok(Tunnel {
            config: config.map(|c| c.0),
            firewall,
            dns_monitor,
            route_manager_handle,
        })
    }
}

pub fn start_tunnel(
    tunnel: &Tunnel,
    tunnel_close_rx: Receiver<()>,
    finished_shutdown_tx: Sender<()>,
) -> Result<JoinHandle<Result<(), crate::error::Error>>, crate::error::Error> {
    let route_manager = tunnel.route_manager_handle.clone();
    // We only start the tunnel when we have wireguard enabled, and then we have the config
    let config = tunnel.config.as_ref().unwrap().clone();
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
        let resource_dir = std::env::temp_dir().join("nym-wg");
        debug!("Tunnel resource dir: {:?}", resource_dir);
        let args = TunnelArgs {
            runtime: tokio::runtime::Handle::current(),
            resource_dir: &resource_dir,
            on_event: on_tunnel_event,
            tunnel_close_rx,
            tun_provider: Arc::new(Mutex::new(TunProvider::new())),
            retry_attempt: 3,
            route_manager,
        };
        info!("Starting wireguard monitor");
        let monitor = WireguardMonitor::start(config, None, None, args)?;
        debug!("Wireguard monitor started, blocking current thread until shutdown");
        if let Err(e) = monitor.wait() {
            error!("Tunnel disconnected with error {:?}", e);
        } else {
            finished_shutdown_tx
                .send(())
                .map_err(|_| crate::error::Error::OneshotSendError)?;
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
