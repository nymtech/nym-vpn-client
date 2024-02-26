use crate::http::client::{HttpError, HTTP_CLIENT};
use std::env;
use tracing::{debug, error, info};

pub const EXPLORER_API_URL: &str = "https://sandbox-explorer.nymtech.net/api/v1";
pub const GATEWAYS_ENDPOINT: &str = "/gateways";

pub type ExplorerJsonGateway = nym_explorer_api_requests::PrettyDetailedGatewayBond;

pub async fn get_gateways() -> Result<Vec<ExplorerJsonGateway>, HttpError> {
    let explorer_api_url = env::var("EXPLORER_API")
        .map(|url| format!("{}/v1", url))
        .unwrap_or_else(|_| EXPLORER_API_URL.to_string());
    let url = format!("{}{}", explorer_api_url, GATEWAYS_ENDPOINT);

    info!("fetching countries from explorer API [{url}]");
    let res = HTTP_CLIENT.get(url).send().await.map_err(|e| {
        error!("HTTP request GET {GATEWAYS_ENDPOINT} failed: {e}");
        HttpError::RequestError(e.status())
    })?;

    debug!("deserializing json response");
    let json: Vec<ExplorerJsonGateway> = res.json().await.map_err(|e| {
        error!("HTTP request GET {GATEWAYS_ENDPOINT} failed to deserialize json response: {e}");
        HttpError::ResponseError(e.to_string())
    })?;
    Ok(json)
}
