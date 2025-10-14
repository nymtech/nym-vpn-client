use std::time::Duration;
use crate::{
    error::{Result, VpnApiClientError},
    response::ApiUrl,
    str_to_socket_addr,
};
use nym_http_api_client::{Client, FrontPolicy, UserAgent};

/// Builds a fronted HTTP client based on the provided `ApiUrl`.
/// We return error `VpnApiClientError::CreateVpnApiClient` which is a bit misleading.
pub fn build_fronted_http_client(
    api_url: &ApiUrl,
    user_agent: Option<UserAgent>,
    timeout: Option<Duration>,
) -> Result<Client> {
    let base_url: url::Url = api_url
        .url
        .parse()
        .map_err(|_e| VpnApiClientError::InvalidUrl {
            url: api_url.url.clone(),
        })?;

    let domain = base_url.domain().ok_or(VpnApiClientError::InvalidUrl {
        url: api_url.url.clone(),
    })?;

    let mut builder = Client::builder(base_url.clone())
        .map_err(|e| VpnApiClientError::CreateVpnApiClient(Box::new(e)))?;

    if let Some(user_agent) = user_agent {
        builder = builder.with_user_agent(user_agent);
    }
    
    if let Some(timeout) = timeout {
        builder = builder.with_timeout(timeout);
    }

    if let Some(fronts) = api_url.fronts.as_ref()
        && !fronts.is_empty()
    {
        builder = builder.with_fronting(FrontPolicy::OnRetry);
        for front in fronts.iter() {
            let addresses = str_to_socket_addr(front)?;
            builder = builder.resolve_to_addrs(domain, &addresses);
        }
    }

    let client = builder
        .build()
        .map_err(|e| VpnApiClientError::CreateVpnApiClient(Box::new(e)))?;

    Ok(client)
}
