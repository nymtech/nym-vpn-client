// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use nym_routing::{Callback, RouteManagerHandle, get_best_default_route};
use nym_socks5_proxy_ipc::InterfaceAddresses;
use nym_windows::net::{AddressFamily, get_ip_address_for_interface};
use tokio::sync::watch;

pub async fn start() -> watch::Receiver<InterfaceAddresses> {
    let initial = snapshot();
    let (tx, rx) = watch::channel(initial);
    tokio::spawn(monitor_task(tx));
    rx
}

fn query_v4() -> Option<std::net::Ipv4Addr> {
    get_best_default_route(AddressFamily::Ipv4)
        .ok()
        .flatten()
        .and_then(|route| {
            get_ip_address_for_interface(AddressFamily::Ipv4, route.iface)
                .ok()
                .flatten()
        })
        .and_then(|ip| {
            if let IpAddr::V4(v4) = ip {
                Some(v4)
            } else {
                None
            }
        })
}

fn query_v6() -> Option<std::net::Ipv6Addr> {
    get_best_default_route(AddressFamily::Ipv6)
        .ok()
        .flatten()
        .and_then(|route| {
            get_ip_address_for_interface(AddressFamily::Ipv6, route.iface)
                .ok()
                .flatten()
        })
        .and_then(|ip| {
            if let IpAddr::V6(v6) = ip {
                Some(v6)
            } else {
                None
            }
        })
}

/// Re-query both address families synchronously.
fn snapshot() -> InterfaceAddresses {
    InterfaceAddresses {
        v4_addr: query_v4(),
        v6_addr: query_v6(),
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

    // Re-query both address families on every event — simpler and race-free.
    let callback: Callback = Box::new(move |_event, _family| {
        let addrs = snapshot();
        tracing::debug!("Default interface changed; new addrs: {addrs:?}");
        let _ = tx.send(addrs);
    });

    match route_manager
        .add_default_route_change_callback(callback)
        .await
    {
        Ok(_handle) => std::future::pending::<()>().await,
        Err(err) => tracing::warn!("Failed to register default route change callback: {err}"),
    }
}
