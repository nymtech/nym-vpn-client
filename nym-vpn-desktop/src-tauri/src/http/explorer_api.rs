use serde::{Deserialize, Serialize};

pub const EXPLORER_API_URL: &str = "https://sandbox-explorer.nymtech.net/api/v1";
pub const GATEWAYS_ENDPOINT: &str = "/gateways/";

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonLocation {
    pub two_letter_iso_country_code: String,
    pub three_letter_iso_country_code: String,
    pub country_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonGatewayInfo {
    pub identity_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonGateway {
    pub owner: String,
    pub location: Option<JsonLocation>,
    pub gateway: JsonGatewayInfo,
}
