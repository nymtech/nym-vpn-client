// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use nym_vpn_api_client::url_to_socket_addr;

use crate::{Config, error::Result, gateway_client::ResolvedConfig};

pub async fn resolve_config(config: &Config) -> Result<ResolvedConfig> {
    let nyxd_socket_addrs = url_to_socket_addr(config.nyxd_url()).await?;
    let api_socket_addrs = url_to_socket_addr(config.api_url()).await?;
    let nym_vpn_api_socket_addrs = if let Some(vpn_api_url) = config.nym_vpn_api_url() {
        Some(url_to_socket_addr(vpn_api_url).await?)
    } else {
        None
    };

    Ok(ResolvedConfig {
        nyxd_socket_addrs,
        api_socket_addrs,
        nym_vpn_api_socket_addrs,
    })
}

pub fn split_ips(ips: Vec<IpAddr>) -> (Vec<Ipv4Addr>, Vec<Ipv6Addr>) {
    ips.into_iter()
        .fold((vec![], vec![]), |(mut v4, mut v6), ip| {
            match ip {
                IpAddr::V4(ipv4_addr) => v4.push(ipv4_addr),
                IpAddr::V6(ipv6_addr) => v6.push(ipv6_addr),
            }
            (v4, v6)
        })
}
