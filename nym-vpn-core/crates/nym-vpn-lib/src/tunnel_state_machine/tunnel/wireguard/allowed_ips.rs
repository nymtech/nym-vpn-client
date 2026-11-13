// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use const_addrs::net;
use ipnetwork::IpNetwork;

use crate::tunnel_state_machine::TunnelConstants;

// Well optimized list of public networks
// https://www.procustodibus.com/blog/2021/03/wireguard-allowedips-calculator/
const ALLOWED_IP_NETWORKS: [IpNetwork; 55] = [
    net!("1.0.0.0/8"),
    net!("2.0.0.0/7"),
    net!("4.0.0.0/6"),
    net!("8.0.0.0/7"),
    net!("11.0.0.0/8"),
    net!("12.0.0.0/6"),
    net!("16.0.0.0/4"),
    net!("32.0.0.0/3"),
    net!("64.0.0.0/3"),
    net!("96.0.0.0/4"),
    net!("112.0.0.0/5"),
    net!("120.0.0.0/6"),
    net!("124.0.0.0/7"),
    net!("126.0.0.0/8"),
    net!("128.0.0.0/3"),
    net!("160.0.0.0/5"),
    net!("168.0.0.0/8"),
    net!("169.0.0.0/9"),
    net!("169.128.0.0/10"),
    net!("169.192.0.0/11"),
    net!("169.224.0.0/12"),
    net!("169.240.0.0/13"),
    net!("169.248.0.0/14"),
    net!("169.252.0.0/15"),
    net!("169.255.0.0/16"),
    net!("170.0.0.0/7"),
    net!("172.0.0.0/12"),
    net!("172.32.0.0/11"),
    net!("172.64.0.0/10"),
    net!("172.128.0.0/9"),
    net!("173.0.0.0/8"),
    net!("174.0.0.0/7"),
    net!("176.0.0.0/4"),
    net!("192.0.0.0/9"),
    net!("192.128.0.0/11"),
    net!("192.160.0.0/13"),
    net!("192.169.0.0/16"),
    net!("192.170.0.0/15"),
    net!("192.172.0.0/14"),
    net!("192.176.0.0/12"),
    net!("192.192.0.0/10"),
    net!("193.0.0.0/8"),
    net!("194.0.0.0/7"),
    net!("196.0.0.0/6"),
    net!("200.0.0.0/5"),
    net!("208.0.0.0/4"),
    net!("224.0.0.0/4"),
    net!("::/1"),
    net!("8000::/2"),
    net!("c000::/3"),
    net!("e000::/4"),
    net!("f000::/5"),
    net!("f800::/6"),
    net!("fe00::/9"),
    net!("ff00::/8"),
];

/// Returns a list of networks that includes all public networks and in-tunnel gateway IP
pub fn allowed_ips(tunnel_constants: &TunnelConstants) -> Vec<IpNetwork> {
    let mut allowed_ips = Vec::with_capacity(ALLOWED_IP_NETWORKS.len() + 1);
    allowed_ips.push(IpNetwork::from(
        tunnel_constants.private_entry_gateway_address,
    ));
    allowed_ips.extend(ALLOWED_IP_NETWORKS);
    allowed_ips
}
