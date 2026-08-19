// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detects whether another VPN client's tunnel already holds the default
//! route before NymVPN starts a connection attempt.
//!
//! Detection works by structure, not by vendor identity: rather than naming
//! specific VPN products, this counts how many distinct tunnel-type network
//! interfaces currently hold a default-route-shaped routing table entry
//! (`0.0.0.0/0`, or the `0.0.0.0/1` + `128.0.0.0/1` split some VPN clients
//! install instead of replacing `0.0.0.0/0` directly). This check must run
//! before NymVPN installs any of its own routes - at that point NymVPN
//! itself doesn't yet account for any such interface, so even a single one
//! means some other VPN client is capturing all traffic. Checking any later
//! doesn't work: many VPN clients tear their own tunnel down the moment they
//! notice they've lost the default route, which is exactly what happens as
//! soon as NymVPN's own route manager takes it over - destroying the
//! evidence before NymVPN ever reaches a connected (or failed) state.
//!
//! Implemented via `nym_routing::get_default_route_interfaces`, which covers
//! Windows, Linux, and macOS. Not available on mobile platforms, where the
//! OS itself only allows one active VPN configuration at a time, so this is
//! a no-op there.

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
use crate::Conflict;

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub(crate) async fn scan() -> Vec<Conflict> {
    if tunnel_interfaces_with_default_route().await >= 1 {
        vec![Conflict::CompetingVpn]
    } else {
        Vec::new()
    }
}

/// Number of distinct virtual/tunnel interfaces (across both address
/// families) currently holding a default-route-shaped entry. Callers must
/// only run this before NymVPN installs any of its own routes for the
/// current attempt - at that point NymVPN doesn't yet account for any such
/// interface, so even one means some other VPN client is capturing all
/// traffic.
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
async fn tunnel_interfaces_with_default_route() -> usize {
    use nym_routing::AddressFamily;

    let mut tunnel_interfaces = std::collections::HashSet::new();

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

#[cfg(all(
    test,
    any(target_os = "windows", target_os = "linux", target_os = "macos")
))]
mod manual_smoke_test {
    #[tokio::test]
    async fn print_scan_result() {
        println!("scan(): {:?}", super::scan().await);
    }
}

/// Synthesizes the "competing VPN" routing signature using `nym-routing`'s
/// own route manager - the same mechanism NymVPN uses to install its own
/// tunnel's default route - instead of requiring a real third-party VPN
/// client to be installed and connected. Detection fires as soon as *any*
/// tunnel interface holds a default route (since this check must only run
/// before NymVPN's own tunnel exists), so this adds a synthetic route on a
/// single idle tunnel adapter to simulate that without it actually being
/// connected. This is `#[ignore]`d because it mutates the system routing
/// table and depends on a specific interface alias being present; run
/// manually with `--ignored`.
#[cfg(all(
    test,
    any(target_os = "windows", target_os = "linux", target_os = "macos")
))]
mod synthesize_competing_vpn {
    use std::{
        collections::HashSet,
        net::{IpAddr, Ipv4Addr},
    };

    use ipnetwork::IpNetwork;
    use nym_routing::{Node, RequiredRoute, RouteManagerHandle};

    // Any already-installed-but-idle tunnel adapter works here since the
    // route just needs to exist in the table - it doesn't need to carry real
    // traffic. Adjust to an interface alias/name present on the test machine.
    #[cfg(target_os = "windows")]
    const SYNTHETIC_TUNNEL_INTERFACE_ALIAS: &str = "Local Area Connection 3";
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    const SYNTHETIC_TUNNEL_INTERFACE_ALIAS: &str = "wg0";

    // Arbitrary, does not need to be reachable - the detector doesn't
    // require a gateway (some VPN clients, e.g. WireGuard-based ones,
    // install an on-link default route with no gateway at all).
    const SYNTHETIC_GATEWAY: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 99, 99, 1));

    #[tokio::test]
    #[ignore = "mutates the system routing table; run manually with --ignored"]
    async fn scan_reports_competing_vpn_for_one_synthetic_default_route() {
        // Linux's route manager additionally needs a fwmark and routing
        // table ID for its policy-routing setup; the values don't matter
        // for this test since we're only adding a plain route to the main
        // table.
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

        let result = super::scan().await;

        route_manager.clear_routes().ok();

        assert!(
            result.contains(&crate::Conflict::CompetingVpn),
            "expected CompetingVpn to be reported once a tunnel interface holds a \
             default route, got: {result:?}"
        );
    }
}
