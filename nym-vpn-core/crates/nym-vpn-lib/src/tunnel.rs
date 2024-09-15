// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashSet,
    path::Path,
    sync::{Arc, Mutex},
};

use futures::channel::{mpsc, oneshot, oneshot::Sender};
use log::*;
use nym_sdk::TaskClient;
use talpid_routing::{RouteManager, RouteManagerHandle};
use talpid_tunnel::{tun_provider::TunProvider, TunnelArgs, TunnelEvent};
use talpid_wireguard::{config::Config, WireguardMonitor};
use tokio::task::JoinHandle;

use crate::wireguard_config::WireguardConfig;

pub(crate) type EventReceiver = mpsc::UnboundedReceiver<(TunnelEvent, Sender<()>)>;

pub(crate) struct Tunnel {
    pub(crate) config: Config,
    pub(crate) route_manager_handle: RouteManagerHandle,
    pub(crate) tun_provider: Arc<Mutex<TunProvider>>,
}

impl Tunnel {
    pub(crate) fn new(
        config: WireguardConfig,
        route_manager_handle: RouteManagerHandle,
        tun_provider: Arc<Mutex<TunProvider>>,
    ) -> Self {
        Tunnel {
            config: config.talpid_config,
            route_manager_handle,
            tun_provider,
        }
    }
}

pub(crate) fn start_tunnel(
    tunnel: &Tunnel,
    mut shutdown: TaskClient,
    finished_shutdown_tx: Sender<()>,
) -> std::result::Result<
    (JoinHandle<()>, EventReceiver, Sender<()>),
    crate::error::SetupWgTunnelError,
> {
    debug!("Starting tunnel");
    let route_manager = tunnel.route_manager_handle.clone();
    // We only start the tunnel when we have wireguard enabled, and then we have the config
    let config = tunnel.config.clone();
    let id: Option<String> = config.tunnel.addresses.first().map(|a| a.to_string());
    let tun_provider = Arc::clone(&tunnel.tun_provider);
    let (event_tx, event_rx) = mpsc::unbounded();
    let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel::<()>();
    let handle = tokio::task::spawn_blocking(move || {
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
        if let Err(e) = std::fs::create_dir_all(&resource_dir) {
            shutdown.send_status_msg(Box::new(e));
            return;
        }
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
        let monitor = match WireguardMonitor::start(
            config,
            None,
            Some(Path::new(&resource_dir.join("logs"))),
            args,
        ) {
            Ok(monitor) => monitor,
            Err(e) => {
                shutdown.send_status_msg(Box::new(e));
                return;
            }
        };
        debug!("Wireguard monitor started, blocking current thread until shutdown");
        if let Err(e) = monitor.wait() {
            error!("Tunnel disconnected with error: {e}");
            shutdown.send_status_msg(Box::new(e));
        } else {
            if finished_shutdown_tx.send(()).is_err() {
                shutdown
                    .send_status_msg(Box::new(crate::error::Error::FailedToSendWireguardShutdown));
            }
            debug!("Sent shutdown message");
        }
        shutdown.disarm();
    });

    Ok((handle, event_rx, tunnel_close_tx))
}

pub(crate) async fn setup_route_manager() -> crate::error::Result<RouteManager> {
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
