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
    let url = Url::new(api_url.url.clone(), api_url.front_hosts.clone()).map_err(|_e| {
        VpnApiClientError::InvalidUrl {
            url: api_url.url.to_string(),
        }
    })?;

    let base_url: url::Url = api_url
        .url
        .parse()
        .map_err(|_e| VpnApiClientError::InvalidUrl {
            url: api_url.url.clone(),
        })?;

    let domain = base_url.domain().ok_or(VpnApiClientError::InvalidUrl {
        url: api_url.url.clone(),
    })?;

    let mut builder = nym_http_api_client::Client::builder(url.clone())
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateVpnApiClient)?;

    if let Some(user_agent) = user_agent {
        builder = builder.with_user_agent(user_agent);
    }

    if let Some(timeout) = timeout {
        builder = builder.with_timeout(timeout);
    }

    if url.has_front() {
        // Have to use ApiUrl fronts as there is no Url::fronts() method :(
        if let Some(fronts) = api_url.front_hosts.as_ref()
            && !fronts.is_empty()
        {
            builder = builder.with_fronting(FrontPolicy::OnRetry);

            for front in fronts.iter() {
                let addresses = str_to_socket_addr(front).await?;
                builder = builder.resolve_to_addrs(domain, &addresses);
            }

            tracing::debug!(
                "Building HTTP client to {} with {} fronts",
                base_url,
                fronts.len()
            );
        }
    } else {
        tracing::debug!("Building HTTP client to {} with no fronts", base_url);
    }

    let client = builder
        .build()
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateVpnApiClient)?;

    Ok(client)
}
