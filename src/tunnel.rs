// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use futures::channel::oneshot::{Receiver, Sender};
use futures::channel::{mpsc, oneshot};
use log::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use talpid_core::dns::DnsMonitor;
use talpid_core::firewall::Firewall;
use talpid_routing::RouteManagerHandle;
use talpid_tunnel::tun_provider::TunProvider;
use talpid_tunnel::TunnelArgs;
use talpid_wireguard::{config::Config, WireguardMonitor};
use tokio::task::JoinHandle;

pub struct Tunnel {
    pub config: Config,
    pub firewall: Firewall,
    pub dns_monitor: DnsMonitor,
    pub route_manager_handle: RouteManagerHandle,
}

impl Tunnel {
    pub fn new(
        config: Config,
        route_manager_handle: RouteManagerHandle,
    ) -> Result<Self, crate::error::Error> {
        let (command_tx, _) = mpsc::unbounded();
        let command_tx = Arc::new(command_tx);
        let weak_command_tx = Arc::downgrade(&command_tx);

        let firewall = Firewall::new()?;
        let dns_monitor = DnsMonitor::new(weak_command_tx)?;

        Ok(Tunnel {
            config,
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
    let config = tunnel.config.clone();
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
        let args = TunnelArgs {
            runtime: tokio::runtime::Handle::current(),
            resource_dir: &PathBuf::from("/tmp/wg-cli"),
            on_event: on_tunnel_event,
            tunnel_close_rx,
            tun_provider: Arc::new(Mutex::new(TunProvider::new())),
            retry_attempt: 3,
            route_manager,
        };
        let monitor = WireguardMonitor::start(config, None, None, args)?;
        info!("Starting wireguard monitor");
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
