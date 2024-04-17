// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    error::Result,
    helpers::{select_random_low_latency_described_gateway, try_resolve_hostname},
    DescribedGatewayWithLocation, Error,
};
use itertools::Itertools;
use nym_explorer_client::{ExplorerClient, Location, PrettyDetailedGatewayBond};
use nym_validator_client::{models::DescribedGateway, NymApiClient};
use std::net::IpAddr;
use tracing::info;
use url::Url;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_url: Url,
    pub explorer_url: Option<Url>,
}

impl Default for Config {
    fn default() -> Self {
        let network_defaults = nym_sdk::NymNetworkDetails::default();
        let default_api_url = network_defaults
            .endpoints
            .first()
            .expect("rust sdk mainnet default incorrectly configured")
            .api_url
            .clone()
            .expect("rust sdk mainnet default missing api_url")
            .parse()
            .expect("rust sdk mainnet default api_url not parseable");
        let default_explorer_url = network_defaults.explorer_api.clone().map(|url| {
            url.parse()
                .expect("rust sdk mainnet default explorer url not parseable")
        });

        Config {
            api_url: default_api_url,
            explorer_url: default_explorer_url,
        }
    }
}

impl Config {
    pub fn api_url(&self) -> &Url {
        &self.api_url
    }

    pub fn with_custom_api_url(mut self, api_url: Url) -> Self {
        self.api_url = api_url;
        self
    }

    pub fn explorer_url(&self) -> Option<&Url> {
        self.explorer_url.as_ref()
    }

    pub fn with_custom_explorer_url(mut self, explorer_url: Url) -> Self {
        self.explorer_url = Some(explorer_url);
        self
    }
}

pub struct GatewayClient {
    api_client: NymApiClient,
    explorer_client: Option<ExplorerClient>,
}

impl GatewayClient {
    pub fn new(config: Config) -> Result<Self> {
        let api_client = NymApiClient::new(config.api_url);
        let explorer_client = if let Some(url) = config.explorer_url {
            Some(ExplorerClient::new(url)?)
        } else {
            None
        };

        Ok(GatewayClient {
            api_client,
            explorer_client,
        })
    }

    async fn lookup_described_gateways(&self) -> Result<Vec<DescribedGateway>> {
        log::info!("Fetching gateways from nym-api...");
        self.api_client
            .get_cached_described_gateways()
            .await
            .map_err(|source| Error::FailedToLookupDescribedGateways { source })
    }

    async fn lookup_gateways_in_explorer(&self) -> Option<Result<Vec<PrettyDetailedGatewayBond>>> {
        log::info!("Fetching gateway geo-locations from nym-explorer...");
        if let Some(explorer_client) = &self.explorer_client {
            Some(
                explorer_client
                    .get_gateways()
                    .await
                    .map_err(|error| Error::FailedFetchLocationData { error }),
            )
        } else {
            None
        }
    }

    pub async fn lookup_described_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways().await?;
        match self.lookup_gateways_in_explorer().await {
            Some(Ok(gateway_locations)) => described_gateways
                .into_iter()
                .map(|gateway| {
                    let location = gateway_locations
                        .iter()
                        .find(|gateway_location| {
                            gateway_location.gateway.identity_key
                                == gateway.bond.gateway.identity_key
                        })
                        .and_then(|gateway_location| gateway_location.location.clone());
                    Ok(DescribedGatewayWithLocation { gateway, location })
                })
                .collect(),
            Some(Err(error)) => {
                // If there was an error fetching the location data, log it and keep on going
                // without location data. This is not a fatal error since we can still refer to the
                // gateways by identity.
                log::warn!("{error}");
                Ok(described_gateways
                    .into_iter()
                    .map(DescribedGatewayWithLocation::from)
                    .collect())
            }
            None => Ok(described_gateways
                .into_iter()
                .map(DescribedGatewayWithLocation::from)
                .collect()),
        }
    }

    pub async fn lookup_described_exit_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter(|gateway| gateway.has_ip_packet_router() && gateway.is_current_build())
            .collect())
    }

    pub async fn lookup_low_latency_entry_gateway(&self) -> Result<DescribedGatewayWithLocation> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        select_random_low_latency_described_gateway(&described_gateways)
            .await
            .cloned()
    }

    pub async fn lookup_all_countries(&self) -> Result<Vec<Location>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.country_name.clone())
            .collect())
    }

    pub async fn lookup_all_countries_iso(&self) -> Result<Vec<Location>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.two_letter_iso_country_code.clone())
            .collect())
    }

    pub async fn lookup_all_exit_countries_iso(&self) -> Result<Vec<Location>> {
        let described_gateways = self.lookup_described_exit_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.two_letter_iso_country_code.clone())
            .collect())
    }

    pub async fn lookup_all_exit_countries(&self) -> Result<Vec<Location>> {
        let described_gateways = self.lookup_described_exit_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.country_name.clone())
            .collect())
    }

    pub async fn lookup_gateway_ip(&self, gateway_identity: &str) -> Result<IpAddr> {
        let ip_or_hostname = self
            .api_client
            .get_cached_gateways()
            .await?
            .iter()
            .find_map(|gateway_bond| {
                if gateway_bond.identity() == gateway_identity {
                    Some(gateway_bond.gateway().host.clone())
                } else {
                    None
                }
            })
            .ok_or(Error::RequestedGatewayIdNotFound(
                gateway_identity.to_string(),
            ))?;

        // If it's a plain IP
        if let Ok(ip) = ip_or_hostname.parse::<IpAddr>() {
            return Ok(ip);
        }

        // If it's not an IP, try to resolve it as a hostname
        let ip = try_resolve_hostname(&ip_or_hostname).await?;
        info!("Resolved {ip_or_hostname} to {ip}");
        Ok(ip)
    }
}
