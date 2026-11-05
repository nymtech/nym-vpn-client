// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use const_addrs::net;
use ipnetwork::IpNetwork;

/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
#[cfg(target_os = "linux")]
pub const SPLIT_TUNNEL_MARK: u32 = 0xf42;

/// Firewall mark used for marking traffic that should bypass the tunnel.
#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x14d;

/// When "allow local network" is enabled the app will allow traffic to and from these networks.
pub const ALLOWED_LAN_NETS: [IpNetwork; 6] = [
    net!("10.0.0.0/8"),
    net!("172.16.0.0/12"),
    net!("192.168.0.0/16"),
    net!("169.254.0.0/16"),
    net!("fe80::/10"),
    net!("fc00::/7"),
];

/// When "allow local network" is enabled the app will allow traffic to these networks.
pub const ALLOWED_LAN_MULTICAST_NETS: [IpNetwork; 8] = [
    // Local network broadcast. Not routable
    net!("255.255.255.255/32"),
    // Local subnetwork multicast. Not routable
    net!("224.0.0.0/24"),
    // Admin-local IPv4 multicast.
    net!("239.0.0.0/8"),
    // Interface-local IPv6 multicast.
    net!("ff01::/16"),
    // Link-local IPv6 multicast. IPv6 equivalent of 224.0.0.0/24
    net!("ff02::/16"),
    // Realm-local IPv6 multicast.
    net!("ff03::/16"),
    // Admin-local IPv6 multicast.
    net!("ff04::/16"),
    // Site-local IPv6 multicast.
    net!("ff05::/16"),
];

pub const LOOPBACK_NETS: [IpNetwork; 2] = [net!("127.0.0.0/8"), net!("::1/128")];

/// Returns whether an address belongs to a private subnet.
pub fn is_local_address(address: &IpAddr) -> bool {
    let address = *address;
    ALLOWED_LAN_NETS
        .iter()
        .chain(&LOOPBACK_NETS)
        .any(|net| net.contains(address))
}
