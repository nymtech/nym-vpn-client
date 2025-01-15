// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use tracing::debug;

use crate::{error::Result, Config, Error, Gateway};

pub(crate) async fn try_resolve_hostname(hostname: &str) -> Result<Vec<IpAddr>> {
    debug!("Trying to resolve hostname: {hostname}");
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let addrs = resolver.lookup_ip(hostname).await.map_err(|err| {
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

#[allow(unused)]
pub async fn allowed_ips(config: &Config, entry_gateway: &Gateway) -> Result<Vec<IpAddr>> {
    let mut ips = entry_gateway.ips.clone();

    let mut nyxd_ips = config.nyxd_ips().await?;
    ips.append(&mut nyxd_ips);

    let mut api_ips = config.api_ips().await?;
    ips.append(&mut api_ips);

    if let Some(mut nym_vpn_api_ips) = config.nym_vpn_api_ips().await? {
        ips.append(&mut nym_vpn_api_ips);
    }

    Ok(ips)
}
