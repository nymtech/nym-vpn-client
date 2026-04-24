// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use crate::default_interface::DefaultInterface;

use nym_routing::{DefaultRouteEvent, RouteManagerHandle};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

pub async fn start_monitor(shutdown_token: CancellationToken) -> watch::Receiver<DefaultInterface> {
    let initial = match RouteManagerHandle::spawn().await {
        Ok(rm) => {
            let addrs = snapshot(&rm).await;
            rm.stop().await;
            addrs
        }
        Err(err) => {
            tracing::warn!("Failed to start route manager to get initial default addrs: {err}");
            DefaultInterface::default()
        }
    };

    let (tx, rx) = watch::channel(initial);
    tokio::spawn(monitor_task(tx, shutdown_token));
    rx
}

async fn snapshot(route_manager: &RouteManagerHandle) -> DefaultInterface {
    match route_manager.get_default_routes().await {
        Ok((v4, v6)) => DefaultInterface {
            v4_addr: v4.and_then(|r| {
                if let IpAddr::V4(a) = r.ip {
                    Some(a)
                } else {
                    None
                }
            }),
            v6_addr: v6.and_then(|r| {
                if let IpAddr::V6(a) = r.ip {
                    Some(a)
                } else {
                    None
                }
            }),
        },
        Err(err) => {
            tracing::warn!("Failed to query default routes: {err}");
            DefaultInterface::default()
        }
    }
}

async fn monitor_task(tx: watch::Sender<DefaultInterface>, shutdown_token: CancellationToken) {
    let route_manager = match RouteManagerHandle::spawn().await {
        Ok(rm) => rm,
        Err(err) => {
            tracing::warn!("Failed to start route manager for default-addr monitor: {err}");
            return;
        }
    };

    let mut listener = match route_manager.default_route_listener().await {
        Ok(rx) => rx,
        Err(err) => {
            tracing::warn!("Failed to subscribe to default route changes: {err}");
            route_manager.stop().await;
            return;
        }
    };

    loop {
        tokio::select! {
            event = listener.recv() => {
                match event {
                    Some(
                        DefaultRouteEvent::AddedOrChangedV4
                        | DefaultRouteEvent::RemovedV4
                        | DefaultRouteEvent::AddedOrChangedV6
                        | DefaultRouteEvent::RemovedV6,
                    ) => {
                        let addrs = snapshot(&route_manager).await;
                        tracing::debug!("Default interface changed; new addrs: {addrs:?}");
                        let _ = tx.send(addrs);
                    }
                    None => break,
                }
            }
            _ = shutdown_token.cancelled() => {
                tracing::debug!("Default interface monitor shutting down");
                break;
            }
        }
    }

    route_manager.stop().await;
}
