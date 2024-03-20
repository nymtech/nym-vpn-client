use reqwest::Response;
use std::env;
use tracing::{debug, error, instrument};

use super::client::HttpError;

pub const NYM_API_URL: &str = "https://sandbox-nym-api1.nymtech.net/api/v1";
pub const GATEWAYS_ENDPOINT: &str = "/gateways/described";

pub type NymApiJsonGateway = nym_api_requests::models::DescribedGateway;

#[instrument]
pub fn get_url() -> String {
    let nym_api_url = env::var("NYM_API")
        .map(|url| format!("{}/v1", url))
        .unwrap_or_else(|_| NYM_API_URL.to_string());
    format!("{}{}", nym_api_url, GATEWAYS_ENDPOINT)
}

#[instrument]
pub async fn deserialize_json(res: Response) -> Result<Vec<NymApiJsonGateway>, HttpError> {
    debug!("deserializing json response");
    let json: Vec<NymApiJsonGateway> = res.json().await.map_err(|e| {
        error!("HTTP request GET {GATEWAYS_ENDPOINT} failed to deserialize json response: {e}");
        HttpError::ResponseError(e.to_string())
    })?;
    Ok(json)
}
