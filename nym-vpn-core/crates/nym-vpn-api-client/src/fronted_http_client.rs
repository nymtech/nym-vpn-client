use crate::error::{Result, VpnApiClientError};
use nym_http_api_client::{Client, FrontPolicy, Url, UserAgent};
use nym_network_defaults::ApiUrl;
use std::time::Duration;

pub async fn build_fronted_http_client(
    api_url: &ApiUrl,
    user_agent: Option<UserAgent>,
    timeout: Option<Duration>,
) -> Result<Client> {
    let url = api_url_to_url(api_url)?;
    let has_front = url.has_front();

    let mut builder = nym_http_api_client::Client::builder(url)
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateVpnApiClient)?;

    if let Some(user_agent) = user_agent {
        builder = builder.with_user_agent(user_agent);
    }

    if let Some(timeout) = timeout {
        builder = builder.with_timeout(timeout);
    }

    if has_front {
        builder = builder.with_fronting(FrontPolicy::OnRetry);
    }

    let client = builder
        .build()
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateVpnApiClient)?;

    Ok(client)
}

pub fn api_url_to_url(api_url: &ApiUrl) -> Result<Url, VpnApiClientError> {
    let parse_url = |s: &str| -> Result<url::Url, VpnApiClientError> {
        match url::Url::parse(s) {
            Ok(url) => Ok(url),
            Err(_) => {
                let with_scheme = format!("http://{s}");
                url::Url::parse(&with_scheme)
                    .map_err(|_e| VpnApiClientError::InvalidUrl { url: s.to_string() })
            }
        }
    };

    let url: url::Url = parse_url(&api_url.url)?;
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
    Url::new(url, fronts).map_err(|_e| VpnApiClientError::InvalidUrl {
        url: api_url.url.to_string(),
    })
}
