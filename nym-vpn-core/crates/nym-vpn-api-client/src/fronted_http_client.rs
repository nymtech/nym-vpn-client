use std::time::Duration;

use crate::{ResolverOverrides, error::VpnApiClientError};
use nym_http_api_client::{Client, ClientBuilder, FrontPolicy, Url, UserAgent};
use nym_network_defaults::ApiUrl;

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
        for domain in resolver_overrides.domains().into_iter() {
            let addrs = resolver_overrides
                .domain_addrs(&domain)
                .expect("Domain was obtained from the overrides");
            tracing::info!(
                "Enabling Resolver override for {domain}: {}",
                addrs
                    .iter()
                    .map(|addr| addr.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            builder = builder.resolve_to_addrs(&domain, &addrs);
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

pub fn api_urls_to_urls(api_urls: &[ApiUrl]) -> Result<Vec<Url>, VpnApiClientError> {
    api_urls
        .iter()
        .map(api_url_to_url)
        .collect::<Result<Vec<_>, _>>()
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
