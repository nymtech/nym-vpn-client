use crate::{
    error::{Result, VpnApiClientError},
    str_to_socket_addr,
};
use nym_http_api_client::{Client, FrontPolicy, Url, UserAgent};
use nym_network_defaults::ApiUrl;
use std::time::Duration;

pub async fn build_fronted_http_client(
    api_url: &ApiUrl,
    user_agent: Option<UserAgent>,
    timeout: Option<Duration>,
) -> Result<Client> {
    let (url, domain) = api_url_to_url(api_url)?;
    let has_front = url.has_front();

    let mut builder = Client::builder(url)
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

        // Have to use ApiUrl fronts as there is no Url::fronts() method :(
        if let Some(fronts) = api_url.front_hosts.as_ref()
            && !fronts.is_empty()
        {
            for front in fronts.iter() {
                let addresses = str_to_socket_addr(front).await?;
                builder = builder.resolve_to_addrs(&domain, &addresses);
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

    let client = builder
        .build()
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateVpnApiClient)?;

    Ok(client)
}

// Returns Ok((url, domain))
pub fn api_url_to_url(api_url: &ApiUrl) -> Result<(Url, String), VpnApiClientError> {
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

    let domain = url
        .domain()
        .ok_or(VpnApiClientError::InvalidUrl {
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
