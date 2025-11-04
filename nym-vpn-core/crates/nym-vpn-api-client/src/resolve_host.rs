use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use futures::stream::{FuturesUnordered, StreamExt};
use nym_common::trace_err_chain;
use nym_http_api_client::HickoryDnsResolver;
use tokio::net::TcpStream;

use crate::error::{Result, VpnApiClientError};

// be generous with the resolution timeout
const HOSTNAME_RESOLUTION_TIMEOUT: Duration = Duration::from_secs(10);

/// DNS resolution timeout for connectivity probes (1s for mobile networks)
const PROBE_DNS_TIMEOUT: Duration = Duration::from_secs(1);

/// TCP connection timeout for connectivity probes (1s for mobile networks)
const PROBE_TCP_TIMEOUT: Duration = Duration::from_secs(1);

/// Overall probe timeout - fail fast if network is not ready
const PROBE_OVERALL_TIMEOUT: Duration = Duration::from_secs(3);

/// Probe targets
const PROBE_TARGETS: &[(&str, u16)] = &[
    ("nymvpn.com", 443),
    ("validator.nymtech.net", 443),
    ("nym.com", 443),
];

async fn try_resolve_hostname(hostname: &str) -> Result<Vec<IpAddr>> {
    tracing::debug!("Trying to resolve hostname: {hostname}");
    let mut resolver = HickoryDnsResolver::default();
    // Disable system resolver because it's typically blocked by firewall anyway.
    resolver.disable_system_fallback();

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
        str_to_socket_addr(&format!("https://{domain}"), limit).await
    }
}

/// Probes connectivity by testing DNS + TCP
/// Returns true on first successful connection, false if all fail or timeout.
pub async fn probe_connectivity() -> bool {
    match tokio::time::timeout(PROBE_OVERALL_TIMEOUT, probe_connectivity_inner()).await {
        Ok(result) => result,
        Err(_) => {
            tracing::warn!(
                "Connectivity probe timed out after {:?}",
                PROBE_OVERALL_TIMEOUT
            );
            false
        }
    }
}

async fn probe_connectivity_inner() -> bool {
    let mut resolver = HickoryDnsResolver::default();
    resolver.disable_system_fallback();

    let mut probe_tasks = FuturesUnordered::new();

    for &(hostname, port) in PROBE_TARGETS {
        let resolver_clone = resolver.clone();
        probe_tasks.push(async move {
            let ips: Vec<IpAddr> =
                match tokio::time::timeout(PROBE_DNS_TIMEOUT, resolver_clone.resolve_str(hostname))
                    .await
                {
                    Ok(Ok(addrs)) => addrs.into_iter().collect(),
                    _ => return false,
                };

            if ips.is_empty() {
                return false;
            }

            let mut connect_tasks = FuturesUnordered::new();
            for ip in ips.into_iter().take(3) {
                let addr = SocketAddr::new(ip, port);
                connect_tasks.push(async move {
                    matches!(
                        tokio::time::timeout(PROBE_TCP_TIMEOUT, TcpStream::connect(addr)).await,
                        Ok(Ok(_))
                    )
                });
            }

            while let Some(success) = connect_tasks.next().await {
                if success {
                    tracing::info!("Connectivity probe succeeded to {hostname}:{port}");
                    return true;
                }
            }
            false
        });
    }

    while let Some(success) = probe_tasks.next().await {
        if success {
            return true;
        }
    }

    tracing::warn!("All connectivity probes failed");
    false
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
