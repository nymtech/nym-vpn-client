// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ipnetwork::IpNetwork;

use crate::tunnel_provider::TunnelSettings;

/// Placeholder interface addresses used while the real tunnel is down (Error / reconnect gap).
pub const BLOCKING_INTERFACE_ADDRS: [IpAddr; 2] = [
    IpAddr::V4(Ipv4Addr::new(169, 254, 0, 10)),
    IpAddr::V6(Ipv6Addr::new(
        0xfdcc, 0x9fc0, 0xe75a, 0x53c3, 0xfa25, 0x241f, 0x21c0, 0x70d0,
    )),
];

/// Minimum IPv6 MTU; used for the blocking placeholder interface.
pub const BLOCKING_TUN_MTU: u16 = 1280;

/// Tunnel settings that keep a VPN interface up without forwarding traffic (blackhole).
pub fn blocking_tunnel_settings(dns_server: IpAddr) -> TunnelSettings {
    TunnelSettings {
        remote_addresses: vec![],
        interface_addresses: BLOCKING_INTERFACE_ADDRS.map(IpNetwork::from).to_vec(),
        dns_servers: vec![dns_server],
        mtu: BLOCKING_TUN_MTU,
        exclude_vpn_app: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocking_tunnel_settings_uses_shared_addrs_and_mtu() {
        let dns = IpAddr::V4(Ipv4Addr::new(169, 254, 0, 10));
        let settings = blocking_tunnel_settings(dns);
        assert_eq!(settings.mtu, BLOCKING_TUN_MTU);
        assert_eq!(settings.dns_servers, vec![dns]);
        assert!(settings.remote_addresses.is_empty());
        // Connecting and reconnect both use this cover; skip-on-cold-connect would leak.
        assert!(settings.exclude_vpn_app);
        assert_eq!(
            settings.interface_addresses,
            BLOCKING_INTERFACE_ADDRS.map(IpNetwork::from).to_vec()
        );
    }
}
