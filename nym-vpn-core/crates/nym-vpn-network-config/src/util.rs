// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

use nym_sdk::NymNetworkDetails;

pub(crate) fn get_age_of_file(file_path: &PathBuf) -> anyhow::Result<Option<Duration>> {
    if !file_path.exists() {
        return Ok(None);
    }
    let metadata = std::fs::metadata(file_path)?;
    Ok(Some(metadata.modified()?.elapsed()?))
}

pub fn resolve_nym_network_details(network_details: &mut NymNetworkDetails) {
    for ep in network_details.endpoints.iter_mut() {
        if let Some(mut url) = ep.api_url() {
            if let Some(sock_addr) = url
                .socket_addrs(|| None)
                .ok()
                .and_then(|sock_addrs| sock_addrs.first().cloned())
            {
                url.set_ip_host(sock_addr.ip()).ok();
                ep.api_url = Some(url.to_string());
            }
        }
        let mut url = ep.nyxd_url();
        if let Some(sock_addr) = url
            .socket_addrs(|| None)
            .ok()
            .and_then(|sock_addrs| sock_addrs.first().cloned())
        {
            url.set_ip_host(sock_addr.ip()).ok();
            ep.nyxd_url = url.to_string();
        }
        if let Some(mut url) = ep.websocket_url() {
            if let Some(sock_addr) = url
                .socket_addrs(|| None)
                .ok()
                .and_then(|sock_addrs| sock_addrs.first().cloned())
            {
                url.set_ip_host(sock_addr.ip()).ok();
                ep.websocket_url = Some(url.to_string());
            }
        }
    }
    if let Some(mut url) = network_details.nym_vpn_api_url() {
        if let Some(sock_addr) = url
            .socket_addrs(|| None)
            .ok()
            .and_then(|sock_addrs| sock_addrs.first().cloned())
        {
            url.set_ip_host(sock_addr.ip()).ok();
            network_details.nym_vpn_api_url = Some(url.to_string());
        }
    }
}

#[cfg(test)]
mod test {
    use url::Host;

    use super::*;

    #[test]
    fn test_resolve_network_details() {
        let mut details = NymNetworkDetails::new_mainnet();
        resolve_nym_network_details(&mut details);
        assert!(matches!(
            details.nym_vpn_api_url().unwrap().host().unwrap(),
            Host::Ipv4(_) | Host::Ipv6(_)
        ));
        for ep in details.endpoints {
            assert!(matches!(
                ep.api_url().unwrap().host().unwrap(),
                Host::Ipv4(_) | Host::Ipv6(_)
            ));
            assert!(matches!(
                ep.nyxd_url().host().unwrap(),
                Host::Ipv4(_) | Host::Ipv6(_)
            ));
            assert!(matches!(
                ep.websocket_url().unwrap().host().unwrap(),
                Host::Ipv4(_) | Host::Ipv6(_)
            ));
        }
    }
}
