use std::env;
use tracing::{debug, error, info};

use crate::http::client::HTTP_CLIENT;

use super::client::HttpError;

pub const NYM_API_URL: &str = "https://sandbox-nym-api1.nymtech.net/api/v1";
pub const GATEWAYS_ENDPOINT: &str = "/gateways/described";

pub type NymApiJsonGateway = nym_api_requests::models::DescribedGateway;

pub async fn get_gateways() -> Result<Vec<NymApiJsonGateway>, HttpError> {
    let nym_api_url = env::var("NYM_API")
        .map(|url| format!("{}/v1", url))
        .unwrap_or_else(|_| NYM_API_URL.to_string());
    let url = format!("{}{}", nym_api_url, GATEWAYS_ENDPOINT);

    info!("fetching countries from Nym API [{url}]");
    let res = HTTP_CLIENT.get(url).send().await.map_err(|e| {
        error!("HTTP request GET {GATEWAYS_ENDPOINT} failed: {e}");
        HttpError::RequestError(e.status())
    })?;

    debug!("deserializing json response");
    let json: Vec<NymApiJsonGateway> = res.json().await.map_err(|e| {
        error!("HTTP request GET {GATEWAYS_ENDPOINT} failed to deserialize json response: {e}");
        HttpError::ResponseError(e.to_string())
    })?;
    Ok(json)
}
