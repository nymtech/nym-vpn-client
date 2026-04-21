// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    io::Error,
    mem::size_of,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    os::windows::io::AsRawSocket,
    ptr::addr_of,
};

use crate::default_interface::DefaultInterface;

use anyhow::{Result, bail};
use nym_routing::{Callback, RouteManagerHandle, get_best_default_route};
use nym_windows::net::{
    AddressFamily, get_best_ipv6_address_for_interface, get_ip_address_for_interface,
    index_from_luid,
};
use tokio::{net::TcpSocket, sync::watch};
use tokio_util::sync::CancellationToken;
use windows::Win32::NetworkManagement::Ndis::NET_LUID_LH;
use windows_sys::Win32::Networking::WinSock::{SOCKET, SOCKET_ERROR, setsockopt};

// These aren't defined by windows-sys
const IPPROTO_IP_LEVEL: i32 = 0; // IPPROTO_IP
const IPPROTO_IPV6_LEVEL: i32 = 41; // IPPROTO_IPV6
const IP_UNICAST_IF_OPT: i32 = 31; // IP_UNICAST_IF  — interface index in network byte order
const IPV6_UNICAST_IF_OPT: i32 = 31; // IPV6_UNICAST_IF — interface index in host byte order

pub async fn start_monitor(shutdown_token: CancellationToken) -> watch::Receiver<DefaultInterface> {
    let initial = snapshot();
    let (tx, rx) = watch::channel(initial);
    tokio::spawn(monitor_task(tx, shutdown_token));
    rx
}

fn query_v4() -> (Option<NET_LUID_LH>, Option<Ipv4Addr>) {
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

fn query_v6() -> (Option<NET_LUID_LH>, Option<Ipv6Addr>) {
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
        Ok(Some(v6)) => (Some(route.iface), Some(v6)),
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

fn snapshot() -> DefaultInterface {
    let (v4_luid, v4_addr) = query_v4();
    let (v6_luid, v6_addr) = query_v6();

    // Complain, but continue, if the interface LUIDs are different.
    if let (Some(v4_luid), Some(v6_luid)) = (v4_luid, v6_luid) {
        unsafe {
            if v4_luid.Value != v6_luid.Value {
                tracing::warn!(
                    "Default IPv4 and IPv6 routes use different LUIDs ({} vs {})!",
                    v4_luid.Value,
                    v6_luid.Value,
                );
            }
        }
    };

    DefaultInterface {
        v4_luid,
        v4_addr,
        v6_luid,
        v6_addr,
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
        Ok(_handle) => {
            // Keep `_handle` alive to maintain the callback registration.
            shutdown_token.cancelled().await;
            tracing::debug!("Default interface monitor shutting down");
        }
        Err(err) => tracing::warn!("Failed to register default route change callback: {err}"),
    }
}

/// Set the interface index on the socket, so it will bind to the default interface.
pub fn set_socket_interface_index(
    socket: &TcpSocket,
    default_interface: &DefaultInterface,
    target_addr: SocketAddr,
) -> Result<()> {
    // Select the LUID that matches the target address family.
    let luid = match target_addr {
        SocketAddr::V4(_) => default_interface.v4_luid,
        SocketAddr::V6(_) => default_interface.v6_luid,
    };

    let Some(luid) = luid else {
        bail!(
            "Cannot bind socket by interface index: no default interface LUID available for {}",
            if target_addr.is_ipv4() {
                "IPv4"
            } else {
                "IPv6"
            }
        );
    };

    let if_index = index_from_luid(&luid)?;

    let raw_socket = socket.as_raw_socket() as SOCKET;

    // IP_UNICAST_IF expects the index in network byte order.
    // IPV6_UNICAST_IF expects the index in host byte order.
    let (ret, opt_name) = match target_addr {
        SocketAddr::V4(_) => {
            let if_index_be = if_index.to_be() as i32;
            let ret = unsafe {
                setsockopt(
                    raw_socket,
                    IPPROTO_IP_LEVEL,
                    IP_UNICAST_IF_OPT,
                    addr_of!(if_index_be) as *const u8,
                    size_of::<u32>() as i32,
                )
            };
            (ret, "IP_UNICAST_IF")
        }
        SocketAddr::V6(_) => {
            let ret = unsafe {
                setsockopt(
                    raw_socket,
                    IPPROTO_IPV6_LEVEL,
                    IPV6_UNICAST_IF_OPT,
                    addr_of!(if_index) as *const u8,
                    size_of::<u32>() as i32,
                )
            };
            (ret, "IPV6_UNICAST_IF")
        }
    };

    if ret == SOCKET_ERROR {
        bail!("setsockopt({opt_name}) failed: {}", Error::last_os_error());
    }

    Ok(())
}
