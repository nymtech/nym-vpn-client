// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};

use nym_routing::{CallbackMessage, RouteManagerHandle};
use nym_socks5_proxy_ipc::InterfaceAddresses;
use tokio::sync::watch;

pub async fn start_monitor() -> watch::Receiver<InterfaceAddresses> {
    let initial = snapshot();
    let (tx, rx) = watch::channel(initial);
    tokio::spawn(monitor_task(tx));
    rx
}

fn udp_source_addr(dest: std::net::SocketAddr) -> Option<IpAddr> {
    let bind: std::net::SocketAddr = match dest {
        std::net::SocketAddr::V4(_) => (Ipv4Addr::UNSPECIFIED, 0).into(),
        std::net::SocketAddr::V6(_) => (Ipv6Addr::UNSPECIFIED, 0).into(),
    };
    let sock = UdpSocket::bind(bind).ok()?;
    sock.connect(dest).ok()?;
    let local = sock.local_addr().ok()?;
    // Discard UNSPECIFIED — means no route was found.
    match local.ip() {
        IpAddr::V4(v4) if v4 != Ipv4Addr::UNSPECIFIED => Some(IpAddr::V4(v4)),
        IpAddr::V6(v6) if v6 != Ipv6Addr::UNSPECIFIED => Some(IpAddr::V6(v6)),
        _ => None,
    }
}

fn snapshot() -> InterfaceAddresses {
    // Use well-known external addresses as route-lookup targets.
    let v4 = udp_source_addr((Ipv4Addr::new(1, 1, 1, 1), 80).into()).and_then(|ip| {
        if let IpAddr::V4(v4) = ip {
            Some(v4)
        } else {
            None
        }
    });
    let v6 = udp_source_addr("[2606:4700:4700::1111]:80".parse().unwrap()).and_then(|ip| {
        if let IpAddr::V6(v6) = ip {
            Some(v6)
        } else {
            None
        }
    });

    InterfaceAddresses {
        v4_addr: v4,
        v6_addr: v6,
    }
}

async fn monitor_task(tx: watch::Sender<InterfaceAddresses>) {
    // fwmark=0 (no mark), table_id=254 (RT_TABLE_MAIN); monitoring only.
    let route_manager = match RouteManagerHandle::spawn(0, 254).await {
        Ok(rm) => rm,
        Err(err) => {
            tracing::warn!("Failed to start route manager for default-addr monitor: {err}");
            return;
        }
    };

    let mut listener = match route_manager.change_listener().await {
        Ok(rx) => rx,
        Err(err) => {
            tracing::warn!("Failed to subscribe to route changes: {err}");
            route_manager.stop().await;
            return;
        }
    };

    while let Some(msg) = listener.recv().await {
        // Re-query whenever a default route (0.0.0.0/0 or ::/0) appears or disappears.
        let is_default = match &msg {
            CallbackMessage::NewRoute(r) | CallbackMessage::DelRoute(r) => {
                r.prefix.prefix_len() == 0
            }
        };
        if is_default {
            let addrs = snapshot();
            tracing::debug!("Default route changed; new addrs: {addrs:?}");
            let _ = tx.send(addrs);
        }
    }

    route_manager.stop().await;
}
