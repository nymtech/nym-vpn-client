use crate::http::client::{HttpError, HTTP_CLIENT};
use reqwest::Response;
use std::env;
use tracing::{debug, error, info, instrument};

pub const EXPLORER_API_URL: &str = "https://sandbox-explorer.nymtech.net/api/v1";
pub const GATEWAYS_ENDPOINT: &str = "/gateways";

pub type ExplorerJsonGateway = nym_explorer_api_requests::PrettyDetailedGatewayBond;

#[instrument]
pub fn get_url() -> String {
    let explorer_api_url = env::var("EXPLORER_API")
        .map(|url| format!("{}/v1", url))
        .unwrap_or_else(|_| EXPLORER_API_URL.to_string());
    format!("{}{}", explorer_api_url, GATEWAYS_ENDPOINT)
}

#[instrument]
pub async fn get_gateways() -> Result<Response, HttpError> {
    let url = get_url();

    info!("fetching countries from explorer API [{url}]");
    let res = HTTP_CLIENT.get(url).send().await.map_err(|e| {
        error!("HTTP request GET {GATEWAYS_ENDPOINT} failed: {e}");
        HttpError::RequestError(e.status())
    })?;
    Ok(res)
}

#[instrument]
pub async fn deserialize_json(res: Response) -> Result<Vec<ExplorerJsonGateway>, HttpError> {
    debug!("deserializing json response");
    let json: Vec<ExplorerJsonGateway> = res.json().await.map_err(|e| {
        error!("HTTP request GET {GATEWAYS_ENDPOINT} failed to deserialize json response: {e}");
        HttpError::ResponseError(e.to_string())
    })?;
    Ok(json)
}
