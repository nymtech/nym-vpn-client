use anyhow::{anyhow, Result};
use itertools::Itertools;
use nym_gateway_directory::{Config, GatewayClient};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tracing::{error, instrument, warn};

use crate::{country::Country, http::HTTP_CLIENT, node_location::NodeType};

pub static GATEWAY_CLIENT: Lazy<Option<GatewayClient>> = Lazy::new(|| {
    let config = Config::new_from_env();
    GatewayClient::new(config)
        .inspect_err(|e| {
            error!("failed to create gateway client: {}", e);
        })
        .ok()
});

pub const GATEWAY_API_URL: &str = "https://nymvpn.com/api/directory/gateways";
pub const ENTRY_ENDPOINT: &str = "/entry/countries";
pub const EXIT_ENDPOINT: &str = "/exit/countries";

#[instrument]
pub async fn get_low_latency_entry_country() -> Result<Country> {
    let client = GATEWAY_CLIENT
        .as_ref()
        .ok_or(anyhow!("gateway client error"))?;
    let described = client
        .lookup_low_latency_entry_gateway()
        .await
        .inspect_err(|e| {
            error!("failed to query low latency gateway: {}", e);
        })?;
    let country = described
        .location()
        .map(|l| Country {
            name: l.country_name.to_string(),
            code: l.two_letter_iso_country_code.to_string(),
        })
        .ok_or(anyhow!("no location found"))?;

    Ok(country)
}

#[derive(Deserialize)]
struct CountryCodes(Vec<String>);

#[instrument]
pub async fn get_gateway_countries(node_type: NodeType) -> Result<Vec<Country>> {
    let codes: CountryCodes = match node_type {
        NodeType::Entry => HTTP_CLIENT
            .get(format!("{}{}", GATEWAY_API_URL, ENTRY_ENDPOINT))
            .send()
            .await
            .inspect_err(|e| {
                error!("failed to send request to fetch entry country codes: {}", e);
            })?
            .json()
            .await
            .inspect_err(|e| error!("failed to deserialize entry country codes response: {}", e))?,
        NodeType::Exit => HTTP_CLIENT
            .get(format!("{}{}", GATEWAY_API_URL, EXIT_ENDPOINT))
            .send()
            .await
            .inspect_err(|e| {
                error!("failed to send request to fetch exit country codes: {}", e);
            })?
            .json()
            .await
            .inspect_err(|e| error!("failed to deserialize exit country codes response: {}", e))?,
    };

    Ok(codes
        .0
        .iter()
        .filter_map(|code| {
            let country = rust_iso3166::from_alpha2(code).map(|country| Country {
                name: country.name.to_string(),
                code: country.alpha2.to_string(),
            });
            if country.is_none() {
                warn!("unknown country code: {}", code);
            }
            country
        })
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect())
}
