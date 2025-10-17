use crate::{
    error::{Result, VpnApiClientError},
    str_to_socket_addr,
};
use nym_http_api_client::{Client, FrontPolicy, HttpClientError, Url, UserAgent};
use nym_network_defaults::ApiUrl;
use std::time::Duration;

pub async fn build_fronted_http_client(
    api_urls: &[ApiUrl],
    user_agent: Option<UserAgent>,
    timeout: Option<Duration>,
) -> Result<Client> {
    let urls_and_domains: Vec<(Url, String)> = api_urls
        .iter()
        .map(api_url_to_url)
        .collect::<Result<Vec<_>, _>>()?;

    // We wouldn't need to do this if `Url::fronts()` existed.
    #[allow(deprecated)]
    if api_urls.len() != urls_and_domains.len() {
        return Err(VpnApiClientError::CreateVpnApiClient(Box::new(
            HttpClientError::GenericRequestFailure(
                "Some of the Nym VPN API URLs in network details are invalid".to_string(),
            ),
        )));
    }

    let urls = urls_and_domains
        .iter()
        .map(|url| url.0.clone())
        .collect::<Vec<_>>();

    let has_front = urls.iter().any(|url| url.has_front());

    let mut builder = nym_http_api_client::ClientBuilder::new_with_urls(urls);

    if let Some(user_agent) = user_agent {
        builder = builder.with_user_agent(user_agent.clone());
    }

    if let Some(timeout) = timeout {
        builder = builder.with_timeout(timeout);
    }

    if has_front {
        builder = builder.with_fronting(FrontPolicy::OnRetry);

        // Have to use ApiUrl fronts as there is no `Url::fronts()` method :(
        for i in 0..api_urls.len() {
            let domain = &urls_and_domains[i].1;
            let api_url = &api_urls[i];
            if let Some(ref fronts) = api_url.front_hosts {
                for front in fronts.iter() {
                    let addresses = str_to_socket_addr(front, Some(1)).await?;
                    builder = builder.resolve_to_addrs(domain, &addresses);

                    tracing::info!(
                        "Enabling Resolver override for {domain}: {}",
                        addresses
                            .iter()
                            .map(|addr| addr.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
        }
    }

    let client = builder
        .build()
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateVpnApiClient)?;

    Ok(client)
}

// Returns (url, domain)
pub fn api_url_to_url(api_url: &ApiUrl) -> Result<(Url, String), VpnApiClientError> {
    let parse_url = |s: &str| -> Result<url::Url, VpnApiClientError> {
        match url::Url::parse(s) {
            Ok(url) => Ok(url),
            Err(_) => {
                let with_scheme = format!("https://{s}");
                url::Url::parse(&with_scheme)
                    .map_err(|_e| VpnApiClientError::InvalidUrl { url: s.to_string() })
            }
        }
    };

    let url = parse_url(&api_url.url)?;

    // For URLs like "http://127.0.0.1:49675", `domain()` returns `None`.
    let domain = url
        .domain()
        .or(url.host_str())
        .ok_or_else(|| VpnApiClientError::InvalidUrl {
            url: api_url.url.clone(),
        })?
        .to_string();

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

    Ok((http_url, domain))
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
        let (url, domain) = api_url_to_url(&api_url).unwrap();
        assert_eq!(url.as_str(), "https://example.com/api");
        assert_eq!(domain, "example.com");
    }

    #[test]
    fn test_api_url_to_url_ipaddr() {
        let api_url = ApiUrl {
            url: "http://127.0.0.1:49675".to_string(),
            front_hosts: Some(vec!["front1.com".to_string(), "front2.com".to_string()]),
        };
        let (url, domain) = api_url_to_url(&api_url).unwrap();
        assert_eq!(url.as_str(), "http://127.0.0.1:49675/");
        assert_eq!(domain, "127.0.0.1");
    }
}
