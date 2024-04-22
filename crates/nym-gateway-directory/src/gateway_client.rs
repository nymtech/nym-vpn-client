// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    error::Result,
    helpers::{
        filter_on_exit_gateways, filter_on_harbour_master_entry_data,
        filter_on_harbour_master_exit_data, select_random_low_latency_described_gateway,
        try_resolve_hostname,
    },
    DescribedGatewayWithLocation, Error,
};
use itertools::Itertools;
use nym_explorer_client::{ExplorerClient, Location, PrettyDetailedGatewayBond};
use nym_harbour_master_client::{
    Gateway as HmGateway, HarbourMasterApiClientExt, PagedResult as HmPagedResult,
};
use nym_validator_client::{models::DescribedGateway, NymApiClient};
use std::net::IpAddr;
use tracing::{debug, info};
use url::Url;

const MAINNET_HARBOUR_MASTER_URL: &str = "https://harbourmaster.nymtech.net";

#[derive(Clone, Debug)]
pub struct Config {
    pub api_url: Url,
    pub explorer_url: Option<Url>,
    pub harbour_master_url: Option<Url>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new_mainnet()
    }
}

impl Config {
    fn new_mainnet() -> Self {
        let mainnet_network_defaults = nym_sdk::NymNetworkDetails::default();
        let default_api_url = mainnet_network_defaults
            .endpoints
            .first()
            .expect("rust sdk mainnet default incorrectly configured")
            .api_url
            .clone()
            .expect("rust sdk mainnet default missing api_url")
            .parse()
            .expect("rust sdk mainnet default api_url not parseable");
        let default_explorer_url = mainnet_network_defaults.explorer_api.clone().map(|url| {
            url.parse()
                .expect("rust sdk mainnet default explorer url not parseable")
        });
        let default_harbour_master_url = Some(
            MAINNET_HARBOUR_MASTER_URL
                .parse()
                .expect("mainnet default harbour master url not parseable"),
        );

        Config {
            api_url: default_api_url,
            explorer_url: default_explorer_url,
            harbour_master_url: default_harbour_master_url,
        }
    }

    pub fn new_from_env() -> Self {
        let network = nym_sdk::NymNetworkDetails::new_from_env();
        let api_url = network
            .endpoints
            .first()
            .expect("network environment endpoints not correctly configured")
            .api_url
            .clone()
            .expect("network environment missing api_url")
            .parse()
            .expect("network environment api_url not parseable");
        let explorer_url = network.explorer_api.clone().map(|url| {
            url.parse()
                .expect("network environment explorer url not parseable")
        });

        // Since harbourmatser isn't part of the standard nym network details, we need to handle it
        // as a special case.
        let harbour_master_url = if network.network_name == "mainnet" {
            Some(
                MAINNET_HARBOUR_MASTER_URL
                    .parse()
                    .expect("mainnet default harbour master url not parseable"),
            )
        } else {
            std::env::var("HARBOUR_MASTER_URL").ok().map(|url| {
                url.parse()
                    .expect("HARBOUR_MASTER_URL env variable not a valid URL")
            })
        };

        Config {
            api_url,
            explorer_url,
            harbour_master_url,
        }
    }

    // If you want to use a custom API URL, you are _very_ likely to also want to custom URLs
    // for the explorer and harbour master as well.
    pub fn new_from_urls(
        api_url: Url,
        explorer_url: Option<Url>,
        harbour_master_url: Option<Url>,
    ) -> Self {
        Config {
            api_url,
            explorer_url,
            harbour_master_url,
        }
    }

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

    pub fn harbour_master_url(&self) -> Option<&Url> {
        self.harbour_master_url.as_ref()
    }

    pub fn with_custom_harbour_master_url(mut self, harbour_master_url: Url) -> Self {
        self.harbour_master_url = Some(harbour_master_url);
        self
    }
}

pub struct GatewayClient {
    api_client: NymApiClient,
    explorer_client: Option<ExplorerClient>,
    harbour_master_client: Option<nym_harbour_master_client::Client>,
}

impl GatewayClient {
    pub fn new(config: Config) -> Result<Self> {
        let api_client = NymApiClient::new(config.api_url);
        let explorer_client = if let Some(url) = config.explorer_url {
            Some(ExplorerClient::new(url)?)
        } else {
            None
        };
        let harbour_master_client = if let Some(url) = config.harbour_master_url {
            Some(nym_harbour_master_client::Client::new_url(url, None)?)
        } else {
            None
        };

        Ok(GatewayClient {
            api_client,
            explorer_client,
            harbour_master_client,
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

    async fn lookup_gateways_in_harbour_master(&self) -> Option<Result<HmPagedResult<HmGateway>>> {
        log::info!("Fetching gateway status from harbourmaster...");
        if let Some(harbour_master_client) = &self.harbour_master_client {
            Some(
                harbour_master_client
                    .get_gateways()
                    .await
                    .map_err(Error::HarbourMasterApiError),
            )
        } else {
            None
        }
    }

    pub async fn lookup_described_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways().await?;
        let described_gateways_location = match self.lookup_gateways_in_explorer().await {
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
                    DescribedGatewayWithLocation { gateway, location }
                })
                .collect(),
            Some(Err(error)) => {
                // If there was an error fetching the location data, log it and keep on going
                // without location data. This is not a fatal error since we can still refer to the
                // gateways by identity.
                log::warn!("{error}");
                described_gateways
                    .into_iter()
                    .map(DescribedGatewayWithLocation::from)
                    .collect::<Vec<_>>()
            }
            None => described_gateways
                .into_iter()
                .map(DescribedGatewayWithLocation::from)
                .collect(),
        };
        Ok(described_gateways_location)
    }

    pub async fn lookup_described_entry_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        let entry_gateways =
            if let Some(Ok(hm_gateways)) = self.lookup_gateways_in_harbour_master().await {
                filter_on_harbour_master_entry_data(described_gateways, hm_gateways.items)
            } else {
                described_gateways
            };
        Ok(entry_gateways)
    }

    pub async fn lookup_described_exit_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        let exit_gateways = filter_on_exit_gateways(described_gateways);
        let exit_gateways =
            if let Some(Ok(hm_gateways)) = self.lookup_gateways_in_harbour_master().await {
                filter_on_harbour_master_exit_data(exit_gateways, hm_gateways.items)
            } else {
                exit_gateways
            };
        Ok(exit_gateways)
    }

    pub async fn lookup_low_latency_entry_gateway(&self) -> Result<DescribedGatewayWithLocation> {
        debug!("Fetching low latency entry gateway...");
        let described_gateways = self.lookup_described_entry_gateways_with_location().await?;
        select_random_low_latency_described_gateway(&described_gateways)
            .await
            .cloned()
    }

    pub async fn lookup_all_countries(&self) -> Result<Vec<Location>> {
        debug!("Fetching all country names from gateways...");
        let described_gateways = self.lookup_described_entry_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.country_name.clone())
            .collect())
    }

    pub async fn lookup_all_countries_iso(&self) -> Result<Vec<Location>> {
        debug!("Fetching all country ISO codes from gateways...");
        let described_gateways = self.lookup_described_entry_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.two_letter_iso_country_code.clone())
            .collect())
    }

    pub async fn lookup_all_exit_countries_iso(&self) -> Result<Vec<Location>> {
        debug!("Fetching all exit country ISO codes from gateways...");
        let described_gateways = self.lookup_described_exit_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.two_letter_iso_country_code.clone())
            .collect())
    }

    pub async fn lookup_all_exit_countries(&self) -> Result<Vec<Location>> {
        debug!("Fetching all exit country names from gateways...");
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
