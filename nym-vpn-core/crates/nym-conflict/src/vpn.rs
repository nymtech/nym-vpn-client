// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detects whether another VPN client's tunnel already holds the default
//! route before NymVPN starts a connection attempt.

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::Conflict;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::collections::HashSet;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) async fn detect() -> Vec<Conflict> {
    if tunnel_interfaces_with_default_route().await >= 1 {
        vec![Conflict::CompetingVpn]
    } else {
        Vec::new()
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn tunnel_interfaces_with_default_route() -> usize {
    use nym_routing::AddressFamily;

    let mut tunnel_interfaces = HashSet::new();

    for family in [AddressFamily::Ipv4, AddressFamily::Ipv6] {
        match nym_routing::get_default_route_interfaces(family).await {
            Ok(interfaces) => tunnel_interfaces.extend(interfaces.virtual_),
            Err(error) => {
                tracing::debug!("failed to get default route interfaces for {family:?}: {error}");
            }
        }
    }

    tunnel_interfaces.len()
}

#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
mod manual_smoke_test {
    #[tokio::test]
    async fn print_scan_result() {
        println!("scan(): {:?}", super::detect().await);
    }
}

#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
mod synthesize_competing_vpn {
    use std::{
        collections::HashSet,
        net::{IpAddr, Ipv4Addr},
    };

    use ipnetwork::IpNetwork;
    use nym_routing::{Node, RequiredRoute, RouteManagerHandle};

    #[cfg(target_os = "windows")]
    const SYNTHETIC_TUNNEL_INTERFACE_ALIAS: &str = "Local Area Connection 3";
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    const SYNTHETIC_TUNNEL_INTERFACE_ALIAS: &str = "wg0";

    const SYNTHETIC_GATEWAY: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 99, 99, 1));

    #[tokio::test]
    #[ignore = "mutates the system routing table; run manually with --ignored"]
    async fn scan_reports_competing_vpn_for_one_synthetic_default_route() {
        #[cfg(target_os = "linux")]
        let route_manager = RouteManagerHandle::spawn(0xf000, 200)
            .await
            .expect("failed to start route manager");
        #[cfg(not(target_os = "linux"))]
        let route_manager = RouteManagerHandle::spawn()
            .await
            .expect("failed to start route manager");

        let prefix: IpNetwork = "0.0.0.0/0".parse().unwrap();
        let node = Node::new(
            SYNTHETIC_GATEWAY,
            SYNTHETIC_TUNNEL_INTERFACE_ALIAS.to_string(),
        );
        let routes = HashSet::from([RequiredRoute::new(prefix, node)]);

        route_manager
            .add_routes(routes)
            .await
            .expect("failed to add synthetic competing-VPN route");

        let result = super::detect().await;

        route_manager.clear_routes().ok();

        assert!(
            result.contains(&crate::Conflict::CompetingVpn),
            "expected CompetingVpn to be reported once a tunnel interface holds a \
             default route, got: {result:?}"
        );
    }
}
