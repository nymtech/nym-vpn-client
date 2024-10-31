// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use tracing::debug;

use crate::{error::Result, Error};

// I've explicitly left this unused code as it might become relevant later
#[allow(unused)]
pub(crate) async fn try_resolve_hostname(hostname: &str) -> Result<IpAddr> {
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

    // Just pick the first one
    addrs
        .iter()
        .next()
        .ok_or(Error::ResolvedHostnameButNoIp(hostname.to_string()))
}
