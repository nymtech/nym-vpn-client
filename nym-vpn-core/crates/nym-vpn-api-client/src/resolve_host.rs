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

    Ok(try_resolve_hostname(hostname)
        .await?
        .into_iter()
        .map(|ip| SocketAddr::new(ip, port))
        .collect())
}

pub async fn str_to_socket_addr(unresolved_url: &str) -> Result<Vec<SocketAddr>> {
    let url = match url::Url::parse(unresolved_url) {
        Ok(url) => url,
        Err(_) => {
            let prefixed = format!("https://{unresolved_url}");
            url::Url::parse(&prefixed).map_err(|_e| VpnApiClientError::InvalidUrl {
                url: unresolved_url.to_string(),
            })?
        }
    };

    url_to_socket_addr(&url).await
}
