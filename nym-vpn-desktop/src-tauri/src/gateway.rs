use anyhow::{anyhow, Result};
use itertools::Itertools;
use nym_vpn_lib::gateway_directory::{Config, GatewayClient};
use once_cell::sync::Lazy;
use tracing::{error, instrument};

use crate::country::Country;

pub static GATEWAY_CLIENT: Lazy<Option<GatewayClient>> = Lazy::new(|| {
    let config = Config::new_from_env();
    let user_agent = nym_bin_common::bin_info!().into();
    GatewayClient::new(config, user_agent)
        .inspect_err(|e| {
            error!("failed to create gateway client: {}", e);
        })
        .ok()
});

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

#[instrument]
pub async fn get_gateway_countries(exit_only: bool) -> Result<Vec<Country>> {
    let client = GATEWAY_CLIENT
        .as_ref()
        .ok_or(anyhow!("gateway client error"))?;
    let locations = match exit_only {
        true => client.lookup_all_exit_countries().await?,
        false => client.lookup_all_countries().await?,
    };
    Ok(locations
        .into_iter()
        .map(|l| Country {
            name: l.country_name,
            code: l.two_letter_iso_country_code,
        })
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect())
}
