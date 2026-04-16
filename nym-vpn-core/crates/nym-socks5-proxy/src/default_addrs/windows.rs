// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_routing::{Callback, RouteManagerHandle, get_best_default_route};
use nym_socks5_proxy_ipc::InterfaceAddresses;
use nym_windows::net::{
    AddressFamily, get_best_ipv6_address_for_interface, get_ip_address_for_interface,
};
use std::net::IpAddr;
use tokio::sync::watch;

pub async fn start() -> watch::Receiver<InterfaceAddresses> {
    let initial = snapshot();
    let (tx, rx) = watch::channel(initial);
    tokio::spawn(monitor_task(tx));
    rx
}

fn query_v4() -> Option<std::net::Ipv4Addr> {
    let route = match get_best_default_route(AddressFamily::Ipv4) {
        Ok(Some(r)) => r,
        Ok(None) => {
            tracing::debug!("No IPv4 default route found");
            return None;
        }
        Err(err) => {
            tracing::warn!("get_best_default_route(IPv4) failed: {err}");
            return None;
        }
    };
    match get_ip_address_for_interface(AddressFamily::Ipv4, route.iface) {
        Ok(Some(IpAddr::V4(v4))) => Some(v4),
        Ok(Some(other)) => {
            tracing::warn!("Unexpected address family on IPv4 interface: {other}");
            None
        }
        Ok(None) => {
            tracing::debug!("IPv4 default interface has no address");
            None
        }
        Err(err) => {
            tracing::warn!("get_ip_address_for_interface(IPv4) failed: {err}");
            None
        }
    }
}

fn query_v6() -> Option<std::net::Ipv6Addr> {
    let route = match get_best_default_route(AddressFamily::Ipv6) {
        Ok(Some(r)) => r,
        Ok(None) => {
            tracing::debug!(
                "No IPv6 default route found — machine likely has link-local only (fe80::) \
                 with no global IPv6 internet connectivity"
            );
            return None;
        }
        Err(err) => {
            tracing::warn!("get_best_default_route(IPv6) failed: {err}");
            return None;
        }
    };
    match get_best_ipv6_address_for_interface(route.iface) {
        Ok(Some(v6)) => {
            tracing::debug!("Selected IPv6 default interface address: {v6}");
            Some(v6)
        }
        Ok(None) => {
            tracing::debug!(
                "IPv6 default interface has no global unicast address \
                 (only link-local fe80:: addresses present — no global IPv6 connectivity)"
            );
            None
        }
        Err(err) => {
            tracing::warn!("get_best_ipv6_address_for_interface failed: {err}");
            None
        }
    }
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
