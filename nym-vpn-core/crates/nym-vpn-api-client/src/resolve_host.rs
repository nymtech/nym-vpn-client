// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, SocketAddr};

use nym_common::trace_err_chain;
use nym_http_api_client::HickoryDnsResolver;

use crate::error::{DnsResolutionFailure, Result, VpnApiClientError};

async fn try_resolve_hostname(hostname: &str) -> Result<Vec<IpAddr>> {
    tracing::debug!("Trying to resolve hostname: {hostname}");
    let resolver: HickoryDnsResolver = HickoryDnsResolver::default();

    let ips = resolver
        .resolve_str(hostname)
        .await
        .map_err(|err| {
            trace_err_chain!(err, "Failed to resolve hostname");
            VpnApiClientError::DnsResolution {
                failures: vec![DnsResolutionFailure {
                    hostname: hostname.to_owned(),
                    source: err,
                }],
            }
        })?
        .collect::<Vec<_>>();

    tracing::debug!("Resolved {hostname} to: {ips:?}");
    // Safety: `HickoryDnsResolver::resolve_str()` always returns at least one IP address on success, otherwise an error.
    assert!(!ips.is_empty());
    Ok(ips)
}

/// Get at most one IPv4 and one IPv6 address of the specified URL.
pub async fn url_to_socket_addr(unresolved_url: &url::Url) -> Result<Vec<SocketAddr>> {
    let port = unresolved_url
        .port_or_known_default()
        .ok_or(VpnApiClientError::UrlError {
            url: unresolved_url.clone(),
            reason: "missing port".to_string(),
        })?;
    let hostname = unresolved_url
        .host_str()
        .ok_or(VpnApiClientError::UrlError {
            url: unresolved_url.clone(),
            reason: "missing hostname".to_string(),
        })?;

    let ips = try_resolve_hostname(hostname).await?;
    let ipv4 = ips.iter().find(|ip| ip.is_ipv4());
    let ipv6 = ips.iter().find(|ip| ip.is_ipv6());

    Ok(ipv4
        .into_iter()
        .chain(ipv6)
        .map(|ip| SocketAddr::new(*ip, port))
        .collect())
}

/// Get at most one IPv4 and one IPv6 address of the specified URL.
pub async fn str_to_socket_addr(unresolved_url: &str) -> Result<Vec<SocketAddr>> {
    let url = url::Url::parse(unresolved_url).map_err(|_e| VpnApiClientError::InvalidUrl {
        url: unresolved_url.to_string(),
    })?;
    url_to_socket_addr(&url).await
}

/// Get at most one IPv4 and one IPv6 address of the specified domain.
pub async fn domain_to_socket_addr(domain: &str) -> Result<Vec<SocketAddr>> {
    if domain.contains("://") {
        str_to_socket_addr(domain).await
    } else {
        str_to_socket_addr(&format!("https://{domain}")).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_resolve_host() {
        let limited_addresses = str_to_socket_addr("https://microsoft.com").await.unwrap();
        assert_eq!(limited_addresses.len(), 2);
        let v4_count = limited_addresses
            .iter()
            .filter(|a| a.ip().is_ipv4())
            .count();
        let v6_count = limited_addresses
            .iter()
            .filter(|a| a.ip().is_ipv6())
            .count();
        assert_eq!(v4_count, 1);
        assert_eq!(v6_count, 1);
    }
}
