// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    entries::gateway::{Gateway, GatewayList},
    error::Result,
    helpers::{
        // filter_on_exit_gateways, filter_on_harbour_master_entry_data,
        // filter_on_harbour_master_exit_data,
        select_random_low_latency_described_gateway,
        try_resolve_hostname,
    },
    AuthAddress, DescribedGatewayWithLocation, Error, IpPacketRouterAddress,
};
// use itertools::Itertools;
use nym_config::defaults;
// use nym_explorer_client::{
// ExplorerClient,
// Location,
// PrettyDetailedGatewayBond,
// };
// use nym_harbour_master_client::{Gateway as HmGateway, HarbourMasterApiClientExt};
use nym_sdk::{mixnet::Recipient, UserAgent};
use nym_topology::IntoGatewayNode;
use nym_validator_client::{models::DescribedGateway, NymApiClient};
use nym_vpn_api_client::VpnApiClientExt;
use std::{fmt, net::IpAddr, time::Duration};
use tracing::{debug, info, warn};
use url::Url;

// const MAINNET_HARBOUR_MASTER_URL: &str = "https://harbourmaster.nymtech.net";
// const HARBOUR_MASTER_CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct GatewayQueryResult {
    pub entry_gateways: Vec<DescribedGatewayWithLocation>,
    pub exit_gateways: Vec<DescribedGatewayWithLocation>,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub api_url: Url,
    // pub explorer_url: Option<Url>,
    // pub harbour_master_url: Option<Url>,
    pub nym_vpn_api_url: Option<Url>,
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
            "api_url: {}, nym_vpn_api_url: {}",
            self.api_url,
            to_string(&self.nym_vpn_api_url),
        )
    }
}

impl Config {
    fn new_mainnet() -> Self {
        let mainnet_network_defaults = nym_sdk::NymNetworkDetails::default();
        let default_api_url = mainnet_network_defaults
            .endpoints
            .first()
            .expect("rust sdk mainnet default incorrectly configured")
            .api_url()
            .expect("rust sdk mainnet default api_url not parseable");
        // let default_explorer_url = mainnet_network_defaults.explorer_api.clone().map(|url| {
        //     url.parse()
        //         .expect("rust sdk mainnet default explorer url not parseable")
        // });
        // let default_harbour_master_url = Some(
        //     MAINNET_HARBOUR_MASTER_URL
        //         .parse()
        //         .expect("mainnet default harbour master url not parseable"),
        // );

        let default_nym_vpn_api_url = Some(
            nym_vpn_api_client::MAINNET_NYM_VPN_API_URL
                .parse()
                .expect("mainnet default nym-vpn-api url not parseable"),
        );

        Config {
            api_url: default_api_url,
            // explorer_url: default_explorer_url,
            // harbour_master_url: default_harbour_master_url,
            nym_vpn_api_url: default_nym_vpn_api_url,
        }
    }

    pub fn new_from_env() -> Self {
        let network = nym_sdk::NymNetworkDetails::new_from_env();
        let api_url = network
            .endpoints
            .first()
            .expect("network environment endpoints not correctly configured")
            .api_url()
            .expect("network environment api_url not parseable");
        // let explorer_url = network.explorer_api.clone().map(|url| {
        //     url.parse()
        //         .expect("network environment explorer url not parseable")
        // });

        // Since harbourmatser isn't part of the standard nym network details, we need to handle it
        // as a special case.
        // let harbour_master_url = if network.network_name == defaults::mainnet::NETWORK_NAME {
        //     Some(
        //         MAINNET_HARBOUR_MASTER_URL
        //             .parse()
        //             .expect("mainnet default harbour master url not parseable"),
        //     )
        // } else {
        //     std::env::var("HARBOUR_MASTER_URL").ok().map(|url| {
        //         url.parse()
        //             .expect("HARBOUR_MASTER_URL env variable not a valid URL")
        //     })
        // };

        let nym_vpn_api_url = if network.network_name == defaults::mainnet::NETWORK_NAME {
            Some(
                nym_vpn_api_client::MAINNET_NYM_VPN_API_URL
                    .parse()
                    .expect("mainnet default nym-vpn-api url not parseable"),
            )
        } else {
            std::env::var("NYM_VPN_API_URL").ok().map(|url| {
                url.parse()
                    .expect("NYM_VPN_API_URL env variable not a valid URL")
            })
        };

        Config {
            api_url,
            // explorer_url,
            // harbour_master_url,
            nym_vpn_api_url,
        }
    }

    // If you want to use a custom API URL, you are _very_ likely to also want to use custom URLs
    // for the explorer and harbour master as well.
    pub fn new_from_urls(
        api_url: Url,
        // explorer_url: Option<Url>,
        // harbour_master_url: Option<Url>,
        nym_vpn_api_url: Option<Url>,
    ) -> Self {
        Config {
            api_url,
            // explorer_url,
            // harbour_master_url,
            nym_vpn_api_url,
        }
    }

    pub fn api_url(&self) -> &Url {
        &self.api_url
    }

    pub fn with_custom_api_url(mut self, api_url: Url) -> Self {
        self.api_url = api_url;
        self
    }

    // pub fn explorer_url(&self) -> Option<&Url> {
    //     self.explorer_url.as_ref()
    // }
    //
    // pub fn with_custom_explorer_url(mut self, explorer_url: Url) -> Self {
    //     self.explorer_url = Some(explorer_url);
    //     self
    // }
    //
    // pub fn harbour_master_url(&self) -> Option<&Url> {
    //     self.harbour_master_url.as_ref()
    // }
    //
    // pub fn with_custom_harbour_master_url(mut self, harbour_master_url: Url) -> Self {
    //     self.harbour_master_url = Some(harbour_master_url);
    //     self
    // }

    pub fn nym_vpn_api_url(&self) -> Option<&Url> {
        self.nym_vpn_api_url.as_ref()
    }

    pub fn with_custom_nym_vpn_api_url(mut self, nym_vpn_api_url: Url) -> Self {
        self.nym_vpn_api_url = Some(nym_vpn_api_url);
        self
    }
}

pub struct GatewayClient {
    api_client: NymApiClient,
    // explorer_client: Option<ExplorerClient>,
    // harbour_master_client: Option<nym_harbour_master_client::Client>,
    nym_vpn_api_client: Option<nym_vpn_api_client::Client>,
}

impl GatewayClient {
    pub fn new(config: Config, user_agent: UserAgent) -> Result<Self> {
        let api_client = NymApiClient::new_with_user_agent(config.api_url, user_agent.clone());
        // let explorer_client = if let Some(url) = config.explorer_url {
        //     Some(ExplorerClient::new(url)?)
        // } else {
        //     None
        // };
        // let harbour_master_client = if let Some(url) = config.harbour_master_url {
        //     Some(nym_harbour_master_client::Client::new_url(
        //         url,
        //         Some(HARBOUR_MASTER_CLIENT_TIMEOUT),
        //     )?)
        // } else {
        //     None
        // };

        let nym_vpn_api_client = if let Some(url) = config.nym_vpn_api_url {
            Some(
                nym_vpn_api_client::ClientBuilder::new(url)
                    .map_err(nym_vpn_api_client::VpnApiClientError::from)?
                    .with_user_agent(user_agent)
                    .with_timeout(Duration::from_secs(10))
                    .build()?,
            )
        } else {
            None
        };

        Ok(GatewayClient {
            api_client,
            // explorer_client,
            // harbour_master_client,
            nym_vpn_api_client,
        })
    }

    pub async fn lookup_described_gateways(&self) -> Result<Vec<DescribedGateway>> {
        info!("Fetching gateways from nym-api...");
        self.api_client
            .get_cached_described_gateways()
            .await
            .map_err(|source| Error::FailedToLookupDescribedGateways { source })
    }

    // async fn lookup_gateways_in_explorer(&self) -> Option<Result<Vec<PrettyDetailedGatewayBond>>> {
    //     if let Some(explorer_client) = &self.explorer_client {
    //         info!("Fetching gateway geo-locations from nym-explorer...");
    //         Some(
    //             explorer_client
    //                 .get_gateways()
    //                 .await
    //                 .map_err(|error| Error::FailedFetchLocationData { error }),
    //         )
    //     } else {
    //         info!("Explorer not configured, skipping...");
    //         None
    //     }
    // }
    //
    // async fn lookup_gateways_in_harbour_master(&self) -> Option<Result<Vec<HmGateway>>> {
    //     if let Some(harbour_master_client) = &self.harbour_master_client {
    //         info!("Fetching gateway status from harbourmaster...");
    //         let gateways = HarbourMasterApiClientExt::get_gateways(harbour_master_client)
    //             .await
    //             .map_err(Error::HarbourMasterApiError);
    //         Some(gateways)
    //     } else {
    //         info!("Harbourmaster not configured, skipping...");
    //         None
    //     }
    // }

    // pub async fn lookup_described_gateways_with_location(
    //     &self,
    // ) -> Result<Vec<DescribedGatewayWithLocation>> {
    //     let described_gateways = self.lookup_described_gateways().await?;
    //     debug!("Got {} gateways from nym-api", described_gateways.len());
    //     let described_gateways_location = match self.lookup_gateways_in_explorer().await {
    //         Some(Ok(gateway_locations)) => {
    //             debug!(
    //                 "Got {} gateway locations from nym-explorer",
    //                 gateway_locations.len()
    //             );
    //             described_gateways
    //                 .into_iter()
    //                 .map(|gateway| {
    //                     let location = gateway_locations
    //                         .iter()
    //                         .find(|gateway_location| {
    //                             gateway_location.gateway.identity_key
    //                                 == gateway.bond.gateway.identity_key
    //                         })
    //                         .and_then(|gateway_location| gateway_location.location.clone());
    //                     DescribedGatewayWithLocation { gateway, location }
    //                 })
    //                 .collect()
    //         }
    //         Some(Err(error)) => {
    //             // If there was an error fetching the location data, log it and keep on going
    //             // without location data. This is not a fatal error since we can still refer to the
    //             // gateways by identity.
    //             warn!("{error}");
    //             described_gateways
    //                 .into_iter()
    //                 .map(DescribedGatewayWithLocation::from)
    //                 .collect::<Vec<_>>()
    //         }
    //         None => described_gateways
    //             .into_iter()
    //             .map(DescribedGatewayWithLocation::from)
    //             .collect(),
    //     };
    //     Ok(described_gateways_location)
    // }

    // TODO: deprecated. Use the one that returns both entry and exit gateways instead
    // pub async fn lookup_described_entry_gateways_with_location(
    //     &self,
    // ) -> Result<Vec<DescribedGatewayWithLocation>> {
    //     let described_gateways = self.lookup_described_gateways_with_location().await?;
    //     debug!(
    //         "After merging with geo data, got {} entry gateways",
    //         described_gateways.len()
    //     );
    //     let entry_gateways =
    //         if let Some(Ok(hm_gateways)) = self.lookup_gateways_in_harbour_master().await {
    //             let gateways = filter_on_harbour_master_entry_data(described_gateways, hm_gateways);
    //             debug!(
    //                 "After filtering on harbourmaster data, got {} entry gateways",
    //                 gateways.len()
    //             );
    //             gateways
    //         } else {
    //             described_gateways
    //         };
    //     Ok(entry_gateways)
    // }
    //
    // TODO: deprecated. Use the one that returns both entry and exit gateways instead
    // pub async fn lookup_described_exit_gateways_with_location(
    //     &self,
    // ) -> Result<Vec<DescribedGatewayWithLocation>> {
    //     let described_gateways = self.lookup_described_gateways_with_location().await?;
    //     debug!(
    //         "After merging with geo data, got {} exit gateways",
    //         described_gateways.len()
    //     );
    //     let exit_gateways = filter_on_exit_gateways(described_gateways);
    //     debug!(
    //         "After filtering on exit gateway capability, got {} exit gateways",
    //         exit_gateways.len()
    //     );
    //     let exit_gateways =
    //         if let Some(Ok(hm_gateways)) = self.lookup_gateways_in_harbour_master().await {
    //             let gateways = filter_on_harbour_master_exit_data(exit_gateways, hm_gateways);
    //             debug!(
    //                 "After filtering on harbourmaster data, got {} exit gateways",
    //                 gateways.len()
    //             );
    //             gateways
    //         } else {
    //             exit_gateways
    //         };
    //     Ok(exit_gateways)
    // }
    //
    // pub async fn lookup_described_entry_and_exit_gateways_with_location(
    //     &self,
    // ) -> Result<GatewayQueryResult> {
    //     let all_gateways = self.lookup_described_gateways_with_location().await?;
    //     debug!(
    //         "After merging with geo data, got {} gateways",
    //         all_gateways.len()
    //     );
    //     let exit_gateways = filter_on_exit_gateways(all_gateways.clone());
    //     debug!(
    //         "After filtering on exit gateway capability, got {} exit gateways",
    //         exit_gateways.len()
    //     );
    //
    //     if let Some(Ok(hm_gateways)) = self.lookup_gateways_in_harbour_master().await {
    //         let entry_gateways =
    //             filter_on_harbour_master_entry_data(all_gateways, hm_gateways.clone());
    //         debug!(
    //             "After filtering on harbourmaster data, got {} entry gateways",
    //             entry_gateways.len()
    //         );
    //
    //         let exit_gateways = filter_on_harbour_master_exit_data(exit_gateways, hm_gateways);
    //         debug!(
    //             "After filtering on harbourmaster data, got {} exit gateways",
    //             exit_gateways.len()
    //         );
    //
    //         Ok(GatewayQueryResult {
    //             entry_gateways,
    //             exit_gateways,
    //         })
    //     } else {
    //         Ok(GatewayQueryResult {
    //             entry_gateways: all_gateways,
    //             exit_gateways,
    //         })
    //     }
    // }
    //
    // pub async fn lookup_low_latency_entry_gateway(&self) -> Result<DescribedGatewayWithLocation> {
    //     debug!("Fetching low latency entry gateway...");
    //     let described_gateways = self.lookup_described_entry_gateways_with_location().await?;
    //     select_random_low_latency_described_gateway(&described_gateways)
    //         .await
    //         .cloned()
    // }

    // KEEP THIS
    pub async fn lookup_low_latency_entry_gateway(&self) -> Result<Gateway> {
        debug!("Fetching low latency entry gateway...");
        let gateways = self.lookup_described_gateways().await?;
        let low_latency_gateway: Gateway = select_random_low_latency_described_gateway(&gateways)
            .await
            .cloned()?
            .try_into()?;
        let gateway_list = self.lookup_entry_gateways().await?;
        gateway_list
            .gateway_with_identity(low_latency_gateway.identity())
            .ok_or(Error::NoMatchingGateway)
            .cloned()
    }

    // pub async fn lookup_all_countries(&self) -> Result<Vec<Location>> {
    //     debug!("Fetching all country names from gateways...");
    //     let described_gateways = self.lookup_described_entry_gateways_with_location().await?;
    //     Ok(described_gateways
    //         .into_iter()
    //         .filter_map(|gateway| gateway.location)
    //         .unique_by(|location| location.country_name.clone())
    //         .collect())
    // }
    //
    // pub async fn lookup_all_countries_iso(&self) -> Result<Vec<Location>> {
    //     debug!("Fetching all country ISO codes from gateways...");
    //     let described_gateways = self.lookup_described_entry_gateways_with_location().await?;
    //     Ok(described_gateways
    //         .into_iter()
    //         .filter_map(|gateway| gateway.location)
    //         .unique_by(|location| location.two_letter_iso_country_code.clone())
    //         .collect())
    // }
    //
    // pub async fn lookup_all_exit_countries_iso(&self) -> Result<Vec<Location>> {
    //     debug!("Fetching all exit country ISO codes from gateways...");
    //     let described_gateways = self.lookup_described_exit_gateways_with_location().await?;
    //     Ok(described_gateways
    //         .into_iter()
    //         .filter_map(|gateway| gateway.location)
    //         .unique_by(|location| location.two_letter_iso_country_code.clone())
    //         .collect())
    // }

    // pub async fn lookup_all_exit_countries(&self) -> Result<Vec<Location>> {
    //     debug!("Fetching all exit country names from gateways...");
    //     let described_gateways = self.lookup_described_exit_gateways_with_location().await?;
    //     Ok(described_gateways
    //         .into_iter()
    //         .filter_map(|gateway| gateway.location)
    //         .unique_by(|location| location.country_name.clone())
    //         .collect())
    // }

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

    pub async fn lookup_all_gateways_from_nym_api(&self) -> Result<GatewayList> {
        let gateways = self
            .lookup_described_gateways()
            .await?
            .into_iter()
            .filter_map(|gw| {
                Gateway::try_from(gw)
                    .inspect_err(|err| warn!("Failed to parse gateway: {err}"))
                    .ok()
            })
            .collect();
        Ok(GatewayList::new(gateways))
    }

    pub async fn lookup_entry_gateways(&self) -> Result<GatewayList> {
        let entry_gateways = if let Some(nym_vpn_api_client) = &self.nym_vpn_api_client {
            info!("Fetching entry gateways from nym-vpn-api...");
            let entry_gateways = nym_vpn_api_client.get_entry_gateways().await?;
            let mut entry_gateways: Vec<_> = entry_gateways
                .into_iter()
                .filter_map(|gw| {
                    Gateway::try_from(gw)
                        .inspect_err(|err| warn!("Failed to parse gateway: {err}"))
                        .ok()
                })
                .collect();

            // Lookup the IPR and authenticator addresses from the nym-api as a temporary hack until
            // the nymvpn.com endpoints are updated to also include these fields.
            let described_gateways = self.lookup_described_gateways().await?;
            append_ipr_and_authenticator_addresses(&mut entry_gateways, described_gateways);
            entry_gateways
        } else {
            self.lookup_described_gateways()
                .await?
                .into_iter()
                .filter_map(|gw| {
                    Gateway::try_from(gw)
                        .inspect_err(|err| warn!("Failed to parse gateway: {err}"))
                        .ok()
                })
                .collect()
        };

        Ok(GatewayList::new(entry_gateways))
    }

    pub async fn lookup_exit_gateways(&self) -> Result<GatewayList> {
        let exit_gateways = if let Some(nym_vpn_api_client) = &self.nym_vpn_api_client {
            info!("Fetching exit gateways from nym-vpn-api...");
            let exit_gateways = nym_vpn_api_client.get_exit_gateways().await?;
            let mut exit_gateways: Vec<_> = exit_gateways
                .into_iter()
                .filter_map(|gw| {
                    Gateway::try_from(gw)
                        .inspect_err(|err| warn!("Failed to parse gateway: {err}"))
                        .ok()
                })
                .collect();

            // Lookup the IPR and authenticator addresses from the nym-api as a temporary hack until
            // the nymvpn.com endpoints are updated to also include these fields.
            let described_gateways = self.lookup_described_gateways().await?;
            append_ipr_and_authenticator_addresses(&mut exit_gateways, described_gateways);
            exit_gateways
        } else {
            self.lookup_described_gateways()
                .await?
                .into_iter()
                .filter_map(|gw| {
                    Gateway::try_from(gw)
                        .inspect_err(|err| warn!("Failed to parse gateway: {err}"))
                        .ok()
                })
                .filter(Gateway::has_ipr_address)
                .collect()
        };

        Ok(GatewayList::new(exit_gateways))
    }
}

// Append the IPR and authenticator addresses to the gateways. This is a temporary hack until the
// nymvpn.com endpoints are updated to also include these fields.
fn append_ipr_and_authenticator_addresses(
    gateways: &mut [Gateway],
    described_gateways: Vec<DescribedGateway>,
) {
    for gateway in gateways.iter_mut() {
        if let Some(described_gateway) = described_gateways
            .iter()
            .find(|dg| dg.identity() == gateway.identity().to_base58_string())
        {
            gateway.ipr_address = described_gateway
                .self_described
                .clone()
                .and_then(|d| d.ip_packet_router)
                .map(|ipr| ipr.address)
                .and_then(|address| IpPacketRouterAddress::try_from_base58_string(&address).ok());
            gateway.authenticator_address = described_gateway
                .self_described
                .clone()
                .and_then(|d| d.authenticator)
                .map(|auth| auth.address)
                .and_then(|address| Recipient::try_from_base58_string(address).ok())
                .map(|r| AuthAddress(Some(r)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nym_sdk::UserAgent;

    fn user_agent() -> UserAgent {
        UserAgent {
            application: "test".to_string(),
            version: "0.0.1".to_string(),
            platform: "test".to_string(),
            git_commit: "test".to_string(),
        }
    }

    #[tokio::test]
    async fn lookup_described_gateways() {
        let config = Config::new_mainnet();
        let client = GatewayClient::new(config, user_agent()).unwrap();
        let gateways = client.lookup_described_gateways().await.unwrap();
        assert!(!gateways.is_empty());
    }

    #[tokio::test]
    async fn lookup_gateways_in_nym_vpn_api() {
        let config = Config::new_mainnet();
        let client = GatewayClient::new(config, user_agent()).unwrap();
        let gateways = client.lookup_exit_gateways().await.unwrap();
        assert!(!gateways.is_empty());
    }
}
