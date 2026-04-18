// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_routing::{Callback, RouteManagerHandle, get_best_default_route};
use nym_windows::net::{
    AddressFamily, get_best_ipv6_address_for_interface, get_ip_address_for_interface,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tokio::sync::watch;

pub async fn start_monitor() -> watch::Receiver<DefaultInterface> {
    let initial = snapshot();
    let (tx, rx) = watch::channel(initial);
    tokio::spawn(monitor_task(tx));
    rx
}

fn query_v4() -> (Option<u32>, Option<Ipv4Addr>) {
    let route = match get_best_default_route(AddressFamily::Ipv4) {
        Ok(Some(r)) => r,
        Ok(None) => {
            tracing::debug!("No IPv4 default route found");
            return (None, None);
        }
        Err(err) => {
            tracing::warn!("get_best_default_route(IPv4) failed: {err}");
            return (None, None);
        }
    };
    match get_ip_address_for_interface(AddressFamily::Ipv4, route.iface) {
        Ok(Some(IpAddr::V4(v4))) => (Some(route.iface), Some(v4)),
        Ok(Some(other)) => {
            tracing::warn!("Unexpected address family on IPv4 interface: {other}");
            (None, None)
        }
        Ok(None) => {
            tracing::debug!("IPv4 default interface has no address");
            (None, None)
        }
        Err(err) => {
            tracing::warn!("get_ip_address_for_interface(IPv4) failed: {err}");
            (None, None)
        }
    }
}

fn query_v6() -> (Option<u32>, Option<Ipv6Addr>) {
    let route = match get_best_default_route(AddressFamily::Ipv6) {
        Ok(Some(r)) => r,
        Ok(None) => {
            tracing::debug!(
                "No IPv6 default route found — machine likely has link-local only (fe80::) \
                 with no global IPv6 internet connectivity"
            );
            return (None, None);
        }
        Err(err) => {
            tracing::warn!("get_best_default_route(IPv6) failed: {err}");
            return (None, None);
        }
    };
    match get_best_ipv6_address_for_interface(route.iface) {
        Ok(Some(v6)) => {
            tracing::debug!("Selected IPv6 default interface address: {v6}");
            (Some(route.iface), Some(v6))
        }
        Ok(None) => {
            tracing::warn!(
                "IPv6 default interface has no global unicast address \
                 (only link-local fe80:: addresses present — no global IPv6 connectivity)"
            );
            (None, None)
        }
        Err(err) => {
            tracing::warn!("get_best_ipv6_address_for_interface failed: {err}");
            (None, None)
        }
    }
}

/// Re-query both address families synchronously.
fn snapshot() -> DefaultInterface {
    let (v4_index, v4_addr) = query_v4();
    let (v6_index, v6_addr) = query_v6();

    let index = if let (Some(v4_idx), Some(v6_idx)) = (v4_index, v6_index) {
        if v4_idx == v6_idx {
            Some(v4_idx)
        } else {
            tracing::warn!(
                "Default IPv4 and IPv6 routes use different interfaces ({} vs {}); \
                 interface index will not be set",
                v4_idx,
                v6_idx
            );
            None
        }
    };
    DefaultInterface {
        index,
        v4_addr,
        v6_addr,
    }
}

async fn monitor_task(tx: watch::Sender<DefaultInterface>) {
    let route_manager = match RouteManagerHandle::spawn().await {
        Ok(rm) => rm,
        Err(err) => {
            tracing::warn!("Failed to start route manager for default-addr monitor: {err}");
            return;
        }
    };

    // Re-query both address families on every event — simpler and race-free.
    let callback: Callback = Box::new(move |_event, _family| {
        let default_interface = snapshot();
        tracing::debug!("Default interface changed: {default_interface:?}");
        let _ = tx.send(default_interface);
    });

    match route_manager
        .add_default_route_change_callback(callback)
        .await
    {
        Ok(_handle) => std::future::pending::<()>().await,
        Err(err) => tracing::warn!("Failed to register default route change callback: {err}"),
    }
}
