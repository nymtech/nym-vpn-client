// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};

/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
#[cfg(target_os = "linux")]
pub const SPLIT_TUNNEL_MARK: u32 = 0xf42;

/// Firewall mark used for marking traffic that should bypass the tunnel.
#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x14d;

/// When "allow local network" is enabled the app will allow traffic to and from these networks.
pub const ALLOWED_LAN_NETS: [IpNetwork; 6] = [
    v4(Ipv4Addr::new(10, 0, 0, 0), 8),
    v4(Ipv4Addr::new(172, 16, 0, 0), 12),
    v4(Ipv4Addr::new(192, 168, 0, 0), 16),
    v4(Ipv4Addr::new(169, 254, 0, 0), 16),
    v6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10),
    v6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7),
];

/// When "allow local network" is enabled the app will allow traffic to these networks.
pub const ALLOWED_LAN_MULTICAST_NETS: [IpNetwork; 8] = [
    // Local network broadcast. Not routable
    v4(Ipv4Addr::new(255, 255, 255, 255), 32),
    // Local subnetwork multicast. Not routable
    v4(Ipv4Addr::new(224, 0, 0, 0), 24),
    // Admin-local IPv4 multicast.
    v4(Ipv4Addr::new(239, 0, 0, 0), 8),
    // Interface-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff01, 0, 0, 0, 0, 0, 0, 0), 16),
    // Link-local IPv6 multicast. IPv6 equivalent of 224.0.0.0/24
    v6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0), 16),
    // Realm-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff03, 0, 0, 0, 0, 0, 0, 0), 16),
    // Admin-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff04, 0, 0, 0, 0, 0, 0, 0), 16),
    // Site-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 0), 16),
];

const LOOPBACK_NETS: [IpNetwork; 2] = [
    IpNetwork::V4(Ipv4Network::new_checked(Ipv4Addr::new(127, 0, 0, 0), 8).unwrap()),
    IpNetwork::V6(Ipv6Network::new_checked(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 128).unwrap()),
];

/// Returns whether an address belongs to a private subnet.
pub fn is_local_address(address: &IpAddr) -> bool {
    let address = *address;
    ALLOWED_LAN_NETS
        .iter()
        .chain(&LOOPBACK_NETS)
        .any(|net| net.contains(address))
}

// Short-hand for `IpNetwork::V4(Ipv4Network::new_checked(address, prefix).unwrap())`.
const fn v4(address: Ipv4Addr, prefix: u8) -> IpNetwork {
    IpNetwork::V4(Ipv4Network::new_checked(address, prefix).unwrap())
}

// Short-hand for `IpNetwork::V6(Ipv6Network::new_checked(address, prefix).unwrap())`.
const fn v6(address: Ipv6Addr, prefix: u8) -> IpNetwork {
    IpNetwork::V6(Ipv6Network::new_checked(address, prefix).unwrap())
}
