use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use nym_common::trace_err_chain;
use nym_http_api_client::HickoryDnsResolver;

use crate::error::{Result, VpnApiClientError};

// be generous with the resolution timeout
const HOSTNAME_RESOLUTION_TIMEOUT: Duration = Duration::from_secs(10);

async fn try_resolve_hostname(hostname: &str) -> Result<Vec<IpAddr>> {
    tracing::debug!("Trying to resolve hostname: {hostname}");
    let resolver = HickoryDnsResolver::default();

    let addrs =
        match tokio::time::timeout(HOSTNAME_RESOLUTION_TIMEOUT, resolver.resolve_str(hostname))
            .await
        {
            Ok(Ok(addrs)) => addrs,
            Ok(Err(err)) => {
                trace_err_chain!(err, "Failed to resolve hostname");
                return Err(VpnApiClientError::FailedToDnsResolveGateway {
                    hostname: hostname.to_string(),
                    source: err,
                });
            }
            Err(_timeout) => {
                return Err(VpnApiClientError::HostnameResolutionTimeout {
                    hostname: hostname.to_string(),
                });
            }
        };

    tracing::debug!("Resolved to: {addrs:?}");

    let ips = addrs.iter().collect::<Vec<_>>();
    if ips.is_empty() {
        return Err(VpnApiClientError::ResolvedHostnameButNoIp(
            hostname.to_string(),
        ));
    }

    Ok(ips)
}

/// Get the address of the specified URL, potentially limiting the number of IPv4, IPv6 addresses returned.
pub async fn url_to_socket_addr(
    unresolved_url: &url::Url,
    limit: Option<(usize, usize)>,
) -> Result<Vec<SocketAddr>> {
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

    let addresses: Vec<SocketAddr> = try_resolve_hostname(hostname)
        .await?
        .into_iter()
        .map(|ip| SocketAddr::new(ip, port))
        .collect();

    if let Some((v4_limit, v6_limit)) = limit {
        let mut limited_addresses = Vec::new();
        let mut v4_count = 0usize;
        let mut v6_count = 0usize;

        for addr in addresses.into_iter() {
            match addr.ip() {
                IpAddr::V4(_) if v4_count < v4_limit => {
                    limited_addresses.push(addr);
                    v4_count += 1;
                }
                IpAddr::V6(_) if v6_count < v6_limit => {
                    limited_addresses.push(addr);
                    v6_count += 1;
                }
                _ => {}
            }
        }

        Ok(limited_addresses)
    } else {
        Ok(addresses)
    }
}

/// Get the address of the specified URL, potentially limiting the number of IPv4, IPv6 addresses returned.
pub async fn str_to_socket_addr(
    unresolved_url: &str,
    limit: Option<(usize, usize)>,
) -> Result<Vec<SocketAddr>> {
    let url = url::Url::parse(unresolved_url).map_err(|_e| VpnApiClientError::InvalidUrl {
        url: unresolved_url.to_string(),
    })?;
    url_to_socket_addr(&url, limit).await
}

/// Get the address of the specified domain, potentially limiting the number of IPv4, IPv6 addresses returned.
pub async fn domain_to_socket_addr(
    domain: &str,
    limit: Option<(usize, usize)>,
) -> Result<Vec<SocketAddr>> {
    if domain.contains("://") {
        str_to_socket_addr(domain, limit).await
    } else {
        str_to_socket_addr(&format!("https://{domain}"), None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_resolve_host() {
        let addresses = domain_to_socket_addr("microsoft.com", None).await.unwrap();

        let limited_addresses = str_to_socket_addr("https://microsoft.com", Some((1, 1)))
            .await
            .unwrap();
        assert!(addresses.len() > 2);
        assert_eq!(limited_addresses.len(), 2);
    }
}
