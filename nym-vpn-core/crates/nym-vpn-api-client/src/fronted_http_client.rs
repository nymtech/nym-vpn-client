use std::time::Duration;

use crate::error::VpnApiClientError;
use nym_http_api_client::{Client, ClientBuilder, FrontPolicy, FrontingConfig, Url, UserAgent};
use nym_network_defaults::ApiUrl;

// plain, plain, then a front: the rotation position persists across calls
const REQUEST_RETRIES: usize = 2;

pub fn fronted_http_client(
    urls: Vec<Url>,
    user_agent: Option<UserAgent>,
    timeout: Option<Duration>,
) -> Result<Client, VpnApiClientError> {
    let builder = fronted_http_client_builder(urls, user_agent, timeout)?;

    let client = builder
        .build()
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateVpnApiClient)?;

    Ok(client)
}

pub fn fronted_http_client_builder(
    urls: Vec<Url>,
    user_agent: Option<UserAgent>,
    timeout: Option<Duration>,
) -> Result<ClientBuilder, VpnApiClientError> {
    let has_front = urls.iter().any(|url| url.has_front());

    let mut builder = ClientBuilder::new_with_urls(urls)
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateVpnApiClient)?;

    // By default, HTTP client uses a singleton instance of reqwest client and hickory resolver.
    // This is not going to work in test environment since singletons are tied to single runtime and each test case uses a new runtime.
    if option_env!("CI_TESTING")
        .and_then(|s| <bool as std::str::FromStr>::from_str(s).ok())
        .unwrap_or_default()
    {
        builder = builder.no_hickory_dns();
    }

    if let Some(user_agent) = user_agent {
        builder = builder.with_user_agent(user_agent.clone());
    }

    if let Some(timeout) = timeout {
        builder = builder.with_timeout(timeout);
    }

    // Plain relays have no fronts: keep them in rotation once fronting is on.
    if has_front {
        let config = FrontingConfig::new(1, 1)
            .with_include_non_fronted_in_rotation(true)
            .with_disable_fronting_on_non_fronting_success(true);
        builder = builder.with_fronting(Some(FrontPolicy::ConfiguredRetry(config)));
    }

    Ok(builder.with_retries(REQUEST_RETRIES))
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
    use nym_http_api_client::{ApiClient, NO_PARAMS};
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::path};

    #[tokio::test]
    async fn request_rotates_to_next_host_when_first_is_blocked() {
        let server = MockServer::start().await;
        Mock::given(path("/ok"))
            .respond_with(ResponseTemplate::new(200).set_body_json(true))
            .mount(&server)
            .await;
        // rotation swaps only the host, so the refused host shares the relay's port
        let blocked = Url::new(format!("http://[::1]:{}", server.address().port()), None).unwrap();
        let relay = Url::new(server.uri(), None).unwrap();

        let client = fronted_http_client(vec![blocked, relay], None, None).unwrap();

        assert!(
            client
                .get_json::<_, bool, _, _>("ok", NO_PARAMS)
                .await
                .unwrap()
        );
    }

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
