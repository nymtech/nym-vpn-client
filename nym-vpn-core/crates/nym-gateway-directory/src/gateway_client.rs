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
use nym_config::defaults::{mainnet, DEFAULT_NYM_NODE_HTTP_PORT};
use nym_explorer_client::{ExplorerClient, Location, PrettyDetailedGatewayBond};
use nym_harbour_master_client::{Gateway as HmGateway, HarbourMasterApiClientExt};
use nym_node_requests::api::client::{NymNodeApiClientError, NymNodeApiClientExt};
use nym_sdk::NymNetworkDetails;
use nym_validator_client::client::MixNodeDetails;
use nym_validator_client::models::{
    IpPacketRouterDetails, NetworkRequesterDetails, NymNodeDescription,
};
use nym_validator_client::nyxd::contract_traits::MixnetQueryClient;
use nym_validator_client::{models::DescribedGateway, nyxd, NymApiClient, QueryHttpRpcNyxdClient};
use std::{fmt, net::IpAddr, time::Duration};
use time::OffsetDateTime;
use tracing::{debug, info, warn};
use url::Url;

const MAINNET_HARBOUR_MASTER_URL: &str = "https://harbourmaster.nymtech.net";
const HARBOUR_MASTER_CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct GatewayQueryResult {
    pub entry_gateways: Vec<DescribedGatewayWithLocation>,
    pub exit_gateways: Vec<DescribedGatewayWithLocation>,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub rpc_url: Url,
    pub api_url: Url,
    pub explorer_url: Option<Url>,
    pub harbour_master_url: Option<Url>,

    pub network_details: NymNetworkDetails,
}

impl Default for Config {
    fn default() -> Self {
        Self::new_mainnet()
    }
}

fn to_string<T: fmt::Display>(value: &Option<T>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "unset".to_string(),
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "api_url: {}, explorer_url: {}, harbour_master_url: {}",
            self.api_url,
            to_string(&self.explorer_url),
            to_string(&self.harbour_master_url)
        )
    }
}

impl Config {
    fn new_mainnet() -> Self {
        let mainnet_network_defaults = nym_sdk::NymNetworkDetails::default();
        let mainnet_endpoints = mainnet_network_defaults
            .endpoints
            .first()
            .expect("rust sdk mainnet default incorrectly configured");
        let default_api_url = mainnet_endpoints
            .api_url()
            .expect("rust sdk mainnet default missing api_url");
        let default_rpc_url = mainnet_endpoints.nyxd_url();

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
            rpc_url: default_rpc_url,
            api_url: default_api_url,
            explorer_url: default_explorer_url,
            harbour_master_url: default_harbour_master_url,
            network_details: mainnet_network_defaults,
        }
    }

    pub fn new_from_env() -> Self {
        let network = nym_sdk::NymNetworkDetails::new_from_env();
        let env_endpoints = network
            .endpoints
            .first()
            .expect("network environment endpoints not correctly configured");
        let api_url = env_endpoints
            .api_url()
            .expect("network environment missing api_url");
        let rpc_url = env_endpoints.nyxd_url();

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
            rpc_url,
            api_url,
            explorer_url,
            harbour_master_url,
            network_details: network,
        }
    }

    // If you want to use a custom API URL, you are _very_ likely to also want to use custom URLs
    // for the explorer and harbour master as well.
    pub fn new_from_urls(
        rpc_url: Url,
        api_url: Url,
        explorer_url: Option<Url>,
        harbour_master_url: Option<Url>,
    ) -> Self {
        Config {
            rpc_url,
            api_url,
            explorer_url,
            harbour_master_url,
            network_details: Default::default(),
        }
    }

    pub fn rpc_url(&self) -> &Url {
        &self.rpc_url
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
    nyxd_client: QueryHttpRpcNyxdClient,
    api_client: NymApiClient,
    explorer_client: Option<ExplorerClient>,
    harbour_master_client: Option<nym_harbour_master_client::Client>,
}

impl GatewayClient {
    pub fn new(config: Config) -> Result<Self> {
        let nyxd_client = QueryHttpRpcNyxdClient::connect(
            nyxd::Config::try_from_nym_network_details(&config.network_details)?,
            config.rpc_url().as_str(),
        )?;

        let api_client = NymApiClient::new(config.api_url);
        let explorer_client = if let Some(url) = config.explorer_url {
            Some(ExplorerClient::new(url)?)
        } else {
            None
        };
        let harbour_master_client = if let Some(url) = config.harbour_master_url {
            Some(nym_harbour_master_client::Client::new_url::<_, String>(
                url,
                Some(HARBOUR_MASTER_CLIENT_TIMEOUT),
            )?)
        } else {
            None
        };

        Ok(GatewayClient {
            nyxd_client,
            api_client,
            explorer_client,
            harbour_master_client,
        })
    }

    async fn lookup_described_gateways(&self) -> Result<Vec<DescribedGateway>> {
        info!("Fetching gateways from nym-api...");
        self.api_client
            .get_cached_described_gateways()
            .await
            .map_err(|source| Error::FailedToLookupDescribedGateways { source })
    }

    async fn lookup_gateways_in_explorer(&self) -> Option<Result<Vec<PrettyDetailedGatewayBond>>> {
        if let Some(explorer_client) = &self.explorer_client {
            info!("Fetching gateway geo-locations from nym-explorer...");
            Some(
                explorer_client
                    .get_gateways()
                    .await
                    .map_err(|error| Error::FailedFetchLocationData { error }),
            )
        } else {
            info!("Explorer not configured, skipping...");
            None
        }
    }

    async fn lookup_gateways_in_harbour_master(&self) -> Option<Result<Vec<HmGateway>>> {
        if let Some(harbour_master_client) = &self.harbour_master_client {
            info!("Fetching gateway status from harbourmaster...");
            let gateways = harbour_master_client
                .get_gateways()
                .await
                .map_err(Error::HarbourMasterApiError);
            Some(gateways)
        } else {
            info!("Harbourmaster not configured, skipping...");
            None
        }
    }

    pub async fn lookup_current_mixnodes(&self) -> Result<Vec<MixNodeDetails>> {
        self.api_client
            .get_cached_mixnodes()
            .await
            .map_err(Into::into)
    }

    pub async fn manually_lookup_contract_gateway(
        &self,
        identity: &str,
    ) -> Result<DescribedGatewayWithLocation> {
        // long-term those should probably be made into shared library (I just copied the code out of nym-api)
        async fn try_get_client(
            identity: &str,
            gateway_host: &str,
        ) -> Result<nym_node_requests::api::Client> {
            // first try the standard port in case the operator didn't put the node behind the proxy,
            // then default https (443)
            // finally default http (80)
            let addresses_to_try = vec![
                format!("http://{gateway_host}:{DEFAULT_NYM_NODE_HTTP_PORT}"),
                format!("https://{gateway_host}"),
                format!("http://{gateway_host}"),
            ];

            for address in addresses_to_try {
                // if provided host was malformed, no point in continuing
                let client = match nym_node_requests::api::Client::new_url(
                    address,
                    Some(Duration::from_secs(5)),
                ) {
                    Ok(client) => client,
                    Err(err) => {
                        warn!(
                            "gateway {identity} provided a malformed host ({gateway_host}): {err}"
                        );
                        // just help our compiler a bit lol
                        let _: NymNodeApiClientError = err;
                        return Err(Error::ManualLookupFailure);
                    }
                };
                if let Ok(health) = client.get_health().await {
                    if health.status.is_up() {
                        return Ok(client);
                    }
                }
            }

            warn!("gateway {identity} has not exposed its self-described API");
            Err(Error::ManualLookupFailure)
        }

        async fn get_gateway_description(
            identity: &str,
            gateway_host: &str,
        ) -> Result<NymNodeDescription> {
            info!("attempting to manually fetch description of {identity}");
            let client = try_get_client(identity, gateway_host).await?;

            info!(
                "gateway {identity} seems to be reachable on {}",
                client.current_url()
            );

            let host_info = client.get_host_information().await?;
            if !host_info.verify_host_information() {
                warn!("{identity} has incorrectly signed its host information");
                return Err(Error::ManualLookupFailure);
            }

            let build_info = client.get_build_information().await?;

            // this can be an old node that hasn't yet exposed this
            let auxiliary_details = client.get_auxiliary_details().await.inspect_err(|err| {
                warn!("could not obtain auxiliary details of gateway {identity}: {err} is it running an old version?");
            }).unwrap_or_default();

            let websockets = client.get_mixnet_websockets().await?;

            let network_requester = if let Ok(nr) = client.get_network_requester().await {
                let exit_policy = client.get_exit_policy().await?;
                let uses_nym_exit_policy = exit_policy.upstream_source == mainnet::EXIT_POLICY_URL;

                Some(NetworkRequesterDetails {
                    address: nr.address,
                    uses_exit_policy: exit_policy.enabled && uses_nym_exit_policy,
                })
            } else {
                None
            };

            let ip_packet_router = if let Ok(ipr) = client.get_ip_packet_router().await {
                Some(IpPacketRouterDetails {
                    address: ipr.address,
                })
            } else {
                None
            };

            Ok(NymNodeDescription {
                host_information: host_info.data.into(),
                last_polled: OffsetDateTime::now_utc().into(),
                build_information: build_info,
                network_requester,
                ip_packet_router,
                mixnet_websockets: websockets.into(),
                auxiliary_details,
            })
        }

        info!("attempting to fetch gateway {identity} from the mixnet contract");
        let Some(bond) = self
            .nyxd_client
            .get_gateway_bond(identity.to_string())
            .await?
            .gateway
        else {
            return Err(Error::NoMatchingGateway);
        };

        let self_described =
            match get_gateway_description(&bond.gateway.identity_key, &bond.gateway.host).await {
                Ok(description) => Some(description),
                Err(err) => {
                    warn!("failed to lookup gateway description: {err}");
                    None
                }
            };

        Ok(DescribedGatewayWithLocation {
            gateway: DescribedGateway {
                bond,
                self_described,
            },
            // we could pull it from `self_described` it needed?
            location: None,
        })
    }

    pub async fn lookup_described_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways().await?;
        debug!("Got {} gateways from nym-api", described_gateways.len());
        let described_gateways_location = match self.lookup_gateways_in_explorer().await {
            Some(Ok(gateway_locations)) => {
                debug!(
                    "Got {} gateway locations from nym-explorer",
                    gateway_locations.len()
                );
                described_gateways
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
                    .collect()
            }
            Some(Err(error)) => {
                // If there was an error fetching the location data, log it and keep on going
                // without location data. This is not a fatal error since we can still refer to the
                // gateways by identity.
                warn!("{error}");
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

    // TODO: deprecated. Use the one that returns both entry and exit gateways instead
    pub async fn lookup_described_entry_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        debug!(
            "After merging with geo data, got {} entry gateways",
            described_gateways.len()
        );
        let entry_gateways =
            if let Some(Ok(hm_gateways)) = self.lookup_gateways_in_harbour_master().await {
                let gateways = filter_on_harbour_master_entry_data(described_gateways, hm_gateways);
                debug!(
                    "After filtering on harbourmaster data, got {} entry gateways",
                    gateways.len()
                );
                gateways
            } else {
                described_gateways
            };
        Ok(entry_gateways)
    }

    // TODO: deprecated. Use the one that returns both entry and exit gateways instead
    pub async fn lookup_described_exit_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        debug!(
            "After merging with geo data, got {} exit gateways",
            described_gateways.len()
        );
        let exit_gateways = filter_on_exit_gateways(described_gateways);
        debug!(
            "After filtering on exit gateway capability, got {} exit gateways",
            exit_gateways.len()
        );
        let exit_gateways =
            if let Some(Ok(hm_gateways)) = self.lookup_gateways_in_harbour_master().await {
                let gateways = filter_on_harbour_master_exit_data(exit_gateways, hm_gateways);
                debug!(
                    "After filtering on harbourmaster data, got {} exit gateways",
                    gateways.len()
                );
                gateways
            } else {
                exit_gateways
            };
        Ok(exit_gateways)
    }

    pub async fn lookup_described_entry_and_exit_gateways_with_location(
        &self,
    ) -> Result<GatewayQueryResult> {
        let all_gateways = self.lookup_described_gateways_with_location().await?;
        debug!(
            "After merging with geo data, got {} gateways",
            all_gateways.len()
        );
        let exit_gateways = filter_on_exit_gateways(all_gateways.clone());
        debug!(
            "After filtering on exit gateway capability, got {} exit gateways",
            exit_gateways.len()
        );

        if let Some(Ok(hm_gateways)) = self.lookup_gateways_in_harbour_master().await {
            let entry_gateways =
                filter_on_harbour_master_entry_data(all_gateways, hm_gateways.clone());
            debug!(
                "After filtering on harbourmaster data, got {} entry gateways",
                entry_gateways.len()
            );

            let exit_gateways = filter_on_harbour_master_exit_data(exit_gateways, hm_gateways);
            debug!(
                "After filtering on harbourmaster data, got {} exit gateways",
                exit_gateways.len()
            );

            Ok(GatewayQueryResult {
                entry_gateways,
                exit_gateways,
            })
        } else {
            Ok(GatewayQueryResult {
                entry_gateways: all_gateways,
                exit_gateways,
            })
        }
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
