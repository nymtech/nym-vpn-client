// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detects whether another VPN client's tunnel is competing for the default
//! route alongside NymVPN's own.
//!
//! Detection works by structure, not by vendor identity: rather than naming
//! specific VPN products, this counts how many distinct tunnel-type network
//! interfaces currently hold a default-route-shaped routing table entry
//! (`0.0.0.0/0`, or the `0.0.0.0/1` + `128.0.0.0/1` split some VPN clients
//! install instead of replacing `0.0.0.0/0` directly). Once NymVPN itself is
//! connected its own tunnel accounts for exactly one such interface, so a
//! second one means some other VPN client is also trying to capture all
//! traffic.
//!
//! Implemented via `nym_routing::get_default_route_interfaces`, which covers
//! Windows, Linux, and macOS. Not available on mobile platforms, where the
//! OS itself only allows one active VPN configuration at a time, so this is
//! a no-op there.

use crate::Conflict;

pub(crate) async fn scan() -> Vec<Conflict> {
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    {
        if tunnel_interfaces_with_default_route().await >= 2 {
            vec![Conflict::CompetingVpn]
        } else {
            Vec::new()
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Vec::new()
    }
}

/// Number of distinct virtual/tunnel interfaces (across both address
/// families) currently holding a default-route-shaped entry. Once NymVPN
/// itself is connected, its own tunnel accounts for exactly one - a second
/// one means some other VPN client is also trying to capture all traffic.
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
/// tunnel's default route - instead of requiring two real VPN clients (our
/// own plus a third party) to be installed and connected simultaneously.
/// Detection only fires once *two* tunnel interfaces hold a default route
/// (NymVPN's own tunnel is expected to account for one), so this adds
/// synthetic routes on two separate idle tunnel adapters to simulate that
/// without needing either to actually be connected. This is `#[ignore]`d
/// because it mutates the system routing table and depends on specific
/// interface aliases being present; run manually with `--ignored`.
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

    // Any two already-installed-but-idle tunnel adapters work here since the
    // routes just need to exist in the table - they don't need to carry real
    // traffic. Adjust to interface aliases/names present on the test machine.
    #[cfg(target_os = "windows")]
    const SYNTHETIC_TUNNEL_INTERFACE_ALIASES: [&str; 2] =
        ["Local Area Connection 3", "Local Area Connection 2"];
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    const SYNTHETIC_TUNNEL_INTERFACE_ALIASES: [&str; 2] = ["wg0", "wg1"];

    // Arbitrary, does not need to be reachable - real VPN clients always
    // install their default route with a gateway (not on-link), which is
    // what the detector actually checks for.
    const SYNTHETIC_GATEWAY: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 99, 99, 1));

    #[tokio::test]
    #[ignore = "mutates the system routing table; run manually with --ignored"]
    async fn scan_reports_competing_vpn_for_two_synthetic_default_routes() {
        // Linux's route manager additionally needs a fwmark and routing
        // table ID for its policy-routing setup; the values don't matter
        // for this test since we're only adding plain routes to the main
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
        let routes = SYNTHETIC_TUNNEL_INTERFACE_ALIASES
            .into_iter()
            .map(|alias| {
                let node = Node::new(SYNTHETIC_GATEWAY, alias.to_string());
                RequiredRoute::new(prefix, node)
            })
            .collect::<HashSet<_>>();

        route_manager
            .add_routes(routes)
            .await
            .expect("failed to add synthetic competing-VPN routes");

        let result = super::scan().await;

        route_manager.clear_routes().ok();

        assert!(
            result.contains(&crate::Conflict::CompetingVpn),
            "expected CompetingVpn to be reported once two tunnel interfaces hold a \
             default route, got: {result:?}"
        );
    }
}
