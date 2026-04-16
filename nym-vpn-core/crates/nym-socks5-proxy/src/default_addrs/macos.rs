// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use nym_routing::{DefaultRouteEvent, RouteManagerHandle};
use nym_socks5_proxy_ipc::InterfaceAddresses;
use tokio::sync::watch;

pub async fn start_monitor() -> watch::Receiver<InterfaceAddresses> {
    let initial = match RouteManagerHandle::spawn().await {
        Ok(rm) => {
            let addrs = query_addrs(&rm).await;
            rm.stop().await;
            addrs
        }
        Err(err) => {
            tracing::warn!("Failed to start route manager to get initial default addrs: {err}");
            InterfaceAddresses::default()
        }
    };

    let (tx, rx) = watch::channel(initial);
    tokio::spawn(monitor_task(tx));
    rx
}

async fn query_addrs(route_manager: &RouteManagerHandle) -> InterfaceAddresses {
    match route_manager.get_default_routes().await {
        Ok((v4, v6)) => InterfaceAddresses {
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
            InterfaceAddresses::default()
        }
    }
}

async fn monitor_task(tx: watch::Sender<InterfaceAddresses>) {
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

    while let Some(event) = listener.recv().await {
        // Any change to either family — re-query both.
        match event {
            DefaultRouteEvent::AddedOrChangedV4
            | DefaultRouteEvent::RemovedV4
            | DefaultRouteEvent::AddedOrChangedV6
            | DefaultRouteEvent::RemovedV6 => {
                let addrs = query_addrs(&route_manager).await;
                tracing::debug!("Default interface changed; new addrs: {addrs:?}");
                let _ = tx.send(addrs);
            }
        }
    }

    route_manager.stop().await;
}
