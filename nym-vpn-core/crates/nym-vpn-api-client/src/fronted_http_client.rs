use std::{collections::HashMap, net::SocketAddr, time::Duration};

use crate::{error::VpnApiClientError, url_to_socket_addr};
use nym_http_api_client::{Client, ClientBuilder, FrontPolicy, Url, UserAgent};
use nym_network_defaults::ApiUrl;

pub type ResolverOverrides = HashMap<String, Vec<SocketAddr>>;

pub async fn fronted_http_client(
    urls: Vec<Url>,
    user_agent: Option<UserAgent>,
    timeout: Option<Duration>,
    resolver_overrides: Option<&ResolverOverrides>,
) -> Result<Client, VpnApiClientError> {
    let builder =
        fronted_http_client_builder(urls, user_agent, timeout, resolver_overrides).await?;

    let client = builder
        .build()
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateVpnApiClient)?;

    Ok(client)
}

pub async fn fronted_http_client_builder(
    urls: Vec<Url>,
    user_agent: Option<UserAgent>,
    timeout: Option<Duration>,
    resolver_overrides: Option<&ResolverOverrides>,
) -> Result<ClientBuilder, VpnApiClientError> {
    let has_front = urls.iter().any(|url| url.has_front());

    let mut builder = ClientBuilder::new_with_urls(urls);

    if let Some(user_agent) = user_agent {
        builder = builder.with_user_agent(user_agent.clone());
    }

    if let Some(timeout) = timeout {
        builder = builder.with_timeout(timeout);
    }

    if has_front {
        builder = builder.with_fronting(FrontPolicy::OnRetry);
    }

    // Add resolver overrides
    if let Some(resolver_overrides) = resolver_overrides.as_ref()
        && !resolver_overrides.is_empty()
    {
        for (domain, addresses) in resolver_overrides.iter() {
            tracing::info!(
                "Enabling Resolver override for {domain}: {}",
                addresses
                    .iter()
                    .map(|addr| addr.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            builder = builder.resolve_to_addrs(domain, addresses);
        }
    }

    Ok(builder)
}

pub fn api_url_to_url(api_url: &ApiUrl) -> Result<Url, VpnApiClientError> {
    let url = parse_url(&api_url.url)?;

    let fronts: Option<Vec<url::Url>> = api_url
        .front_hosts
        .as_ref()
        .map(|hosts| {
            hosts
                .iter()
                .map(|host| parse_url(host))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;

    let http_url = Url::new(url, fronts).map_err(|_e| VpnApiClientError::InvalidUrl {
        url: api_url.url.to_string(),
    })?;

    Ok(http_url)
}

// Returns (url, Some(domain))
pub fn api_url_to_url_and_domain(
    api_url: &ApiUrl,
) -> Result<(Url, Option<String>), VpnApiClientError> {
    let url = parse_url(&api_url.url)?;

    // For URLs like "http://127.0.0.1:49675", `domain()` returns `None`.
    let domain = url.domain().map(|s| s.to_string());

    let fronts: Option<Vec<url::Url>> = api_url
        .front_hosts
        .as_ref()
        .map(|hosts| {
            hosts
                .iter()
                .map(|api_url| parse_url(api_url))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;

    let http_url = Url::new(url, fronts).map_err(|_e| VpnApiClientError::InvalidUrl {
        url: api_url.url.to_string(),
    })?;

    Ok((http_url, domain))
}

pub async fn urls_to_resolver_overrides(
    urls: &[Url],
) -> Result<ResolverOverrides, VpnApiClientError> {
    let mut overrides = ResolverOverrides::new();

    for url in urls {
        let Some(domain) = url.inner_url().domain() else {
            tracing::warn!(
                "Ignoring API URL '{}' for resolver overrides as it does not have a valid domain",
                url.to_string()
            );
            continue;
        };

        let addresses = url_to_socket_addr(url.inner_url(), Some((1, 1))).await?;
        overrides.insert(domain.to_string(), addresses);

        if let Some(fronts) = url.fronts() {
            for front_url in fronts {
                let Some(front_domain) = front_url.domain() else {
                    tracing::warn!(
                        "Ignoring front host URL '{}' for resolver overrides as it does not have a valid domain",
                        front_url
                    );
                    continue;
                };
                let front_addresses = url_to_socket_addr(front_url, Some((1, 1))).await?;
                overrides.insert(front_domain.to_string(), front_addresses);
            }
        }
    }

    Ok(overrides)
}

pub async fn api_urls_to_resolver_overrides(
    api_urls: &[ApiUrl],
) -> Result<ResolverOverrides, VpnApiClientError> {
    let mut overrides = ResolverOverrides::new();

    for api_url in api_urls {
        let (url, Some(domain)) = api_url_to_url_and_domain(api_url)? else {
            tracing::warn!(
                "Ignoring API URL '{}' for resolver overrides as it does not have a valid domain",
                api_url.url
            );
            continue;
        };

        let addresses = url_to_socket_addr(url.inner_url(), Some((1, 1))).await?;
        overrides.insert(domain, addresses);

        if let Some(fronts) = url.fronts() {
            for front_url in fronts {
                let Some(front_domain) = front_url.domain() else {
                    tracing::warn!(
                        "Ignoring front host URL '{}' for resolver overrides as it does not have a valid domain",
                        front_url
                    );
                    continue;
                };
                let front_addresses = url_to_socket_addr(front_url, Some((1, 1))).await?;
                overrides.insert(front_domain.to_string(), front_addresses);
            }
        }
    }

    Ok(overrides)
}

fn parse_url(s: &str) -> Result<url::Url, VpnApiClientError> {
    match url::Url::parse(s) {
        Ok(url) => Ok(url),
        Err(_) => {
            let with_scheme = format!("https://{s}");
            url::Url::parse(&with_scheme)
                .map_err(|_e| VpnApiClientError::InvalidUrl { url: s.to_string() })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url_to_url_domain() {
        let api_url = ApiUrl {
            url: "example.com/api".to_string(),
            front_hosts: Some(vec!["front1.com".to_string(), "front2.com".to_string()]),
        };
        let (url, domain) = api_url_to_url_and_domain(&api_url).unwrap();
        assert_eq!(url.as_str(), "https://example.com/api");
        assert_eq!(domain, Some("example.com".to_string()));
    }

    #[test]
    fn test_api_url_to_url_ipaddr() {
        let api_url = ApiUrl {
            url: "http://127.0.0.1:49675".to_string(),
            front_hosts: Some(vec!["front1.com".to_string(), "front2.com".to_string()]),
        };
        let (url, domain) = api_url_to_url_and_domain(&api_url).unwrap();
        assert_eq!(url.as_str(), "http://127.0.0.1:49675/");
        assert!(domain.is_none());
    }
}
