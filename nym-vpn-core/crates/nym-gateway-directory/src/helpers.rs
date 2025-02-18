// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, SocketAddr};

use nym_http_api_client::HickoryDnsResolver;
use tracing::debug;

use crate::{error::Result, Config, Error};

async fn try_resolve_hostname(hostname: &str) -> Result<Vec<IpAddr>> {
    debug!("Trying to resolve hostname: {hostname}");
    let resolver = HickoryDnsResolver::default();
    let addrs = resolver.resolve_str(hostname).await.map_err(|err| {
        tracing::error!("Failed to resolve gateway hostname: {}", err);
        Error::FailedToDnsResolveGateway {
            hostname: hostname.to_string(),
            source: err,
        }
    })?;
    debug!("Resolved to: {addrs:?}");

    let ips = addrs.iter().collect::<Vec<_>>();
    if ips.is_empty() {
        return Err(Error::ResolvedHostnameButNoIp(hostname.to_string()));
    }

    Ok(ips)
}

pub async fn allowed_ips(config: &Config) -> Result<Vec<SocketAddr>> {
    let mut socket_addrs = vec![];
    let mut urls = vec![];

    urls.push(config.nyxd_url());
    urls.push(config.api_url());
    if let Some(vpn_api_url) = config.nym_vpn_api_url() {
        urls.push(vpn_api_url);
    }

    for unresolved_url in urls {
        let port = unresolved_url
            .port_or_known_default()
            .ok_or(Error::UrlError {
                url: unresolved_url.clone(),
                reason: "missing port".to_string(),
            })?;
        let hostname = unresolved_url.host_str().ok_or(Error::UrlError {
            url: unresolved_url.clone(),
            reason: "missing hostname".to_string(),
        })?;
        socket_addrs.extend(
            try_resolve_hostname(hostname)
                .await?
                .into_iter()
                .map(|ip| SocketAddr::new(ip, port)),
        );
    }

    Ok(socket_addrs)
}
