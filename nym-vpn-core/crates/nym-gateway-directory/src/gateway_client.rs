// Copyright 2023-2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, net::IpAddr};

use nym_sdk::UserAgent;
use nym_validator_client::{models::NymNodeDescription, nym_nodes::SkimmedNode, NymApiClient};
use nym_vpn_api_client::types::{GatewayMinPerformance, Percent};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::entries::gateway::NymNodeList;
use crate::helpers::try_resolve_hostname;
use crate::{
    entries::{
        country::Country,
        gateway::{Gateway, GatewayList, GatewayType},
    },
    error::Result,
    Error, NymNode,
};

#[derive(Clone, Debug)]
pub struct Config {
    pub nyxd_url: Url,
    pub api_url: Url,
    pub nym_vpn_api_url: Option<Url>,
    pub min_gateway_performance: Option<GatewayMinPerformance>,
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
            "nyxd_url: {}, api_url: {}, nym_vpn_api_url: {}",
            self.nyxd_url,
            self.api_url,
            to_string(&self.nym_vpn_api_url),
        )
    }
}

impl Config {
    fn new_mainnet() -> Self {
        let mainnet_network_defaults = nym_sdk::NymNetworkDetails::default();
        let default_nyxd_url = mainnet_network_defaults
            .endpoints
            .first()
            .expect("rust sdk mainnet default incorrectly configured")
            .nyxd_url();
        let default_api_url = mainnet_network_defaults
            .endpoints
            .first()
            .expect("rust sdk mainnet default incorrectly configured")
            .api_url()
            .expect("rust sdk mainnet default api_url not parseable");

        let default_nym_vpn_api_url = mainnet_network_defaults
            .nym_vpn_api_url()
            .expect("rust sdk mainnet default nym-vpn-api url not parseable");

        Config {
            nyxd_url: default_nyxd_url,
            api_url: default_api_url,
            nym_vpn_api_url: Some(default_nym_vpn_api_url),
            min_gateway_performance: None,
        }
    }

    pub fn new_from_env() -> Self {
        let network = nym_sdk::NymNetworkDetails::new_from_env();
        let nyxd_url = network
            .endpoints
            .first()
            .expect("network environment endpoints not correctly configured")
            .nyxd_url();
        let api_url = network
            .endpoints
            .first()
            .expect("network environment endpoints not correctly configured")
            .api_url()
            .expect("network environment api_url not parseable");

        // The vpn api url is strictly not needed, so skip the expect here
        let nym_vpn_api_url = network.nym_vpn_api_url();

        Config {
            nyxd_url,
            api_url,
            nym_vpn_api_url,
            min_gateway_performance: None,
        }
    }

    async fn url_to_ips(url: &Url) -> Result<Vec<IpAddr>> {
        match url.host() {
            Some(url::Host::Ipv4(addr)) => Ok(vec![addr.into()]),
            Some(url::Host::Ipv6(addr)) => Ok(vec![addr.into()]),
            Some(url::Host::Domain(hostname)) => try_resolve_hostname(hostname).await,
            None => Err(Error::NoHostForUrl(url.clone())),
        }
    }

    pub fn nyxd_url(&self) -> &Url {
        &self.nyxd_url
    }

    pub async fn nyxd_ips(&self) -> Result<Vec<IpAddr>> {
        Self::url_to_ips(&self.nyxd_url).await
    }

    pub fn with_custom_nyxd_url(mut self, nyxd_url: Url) -> Self {
        self.nyxd_url = nyxd_url;
        self
    }

    pub fn api_url(&self) -> &Url {
        &self.api_url
    }

    pub async fn api_ips(&self) -> Result<Vec<IpAddr>> {
        Self::url_to_ips(&self.api_url).await
    }

    pub fn with_custom_api_url(mut self, api_url: Url) -> Self {
        self.api_url = api_url;
        self
    }

    pub fn nym_vpn_api_url(&self) -> Option<&Url> {
        self.nym_vpn_api_url.as_ref()
    }

    pub async fn nym_vpn_api_ips(&self) -> Result<Option<Vec<IpAddr>>> {
        if let Some(url) = &self.nym_vpn_api_url {
            Self::url_to_ips(url).await.map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn with_custom_nym_vpn_api_url(mut self, nym_vpn_api_url: Url) -> Self {
        self.nym_vpn_api_url = Some(nym_vpn_api_url);
        self
    }

    pub fn with_min_gateway_performance(
        mut self,
        min_gateway_performance: GatewayMinPerformance,
    ) -> Self {
        self.min_gateway_performance = Some(min_gateway_performance);
        self
    }
}

pub struct GatewayClient {
    api_client: NymApiClient,
    nym_vpn_api_client: Option<nym_vpn_api_client::VpnApiClient>,
    min_gateway_performance: Option<GatewayMinPerformance>,
}

impl GatewayClient {
    pub fn new(config: Config, user_agent: UserAgent) -> Result<Self> {
        let api_client = NymApiClient::new_with_user_agent(config.api_url, user_agent.clone());
        let nym_vpn_api_client = config
            .nym_vpn_api_url
            .map(|url| nym_vpn_api_client::VpnApiClient::new(url, user_agent.clone()))
            .transpose()?;

        Ok(GatewayClient {
            api_client,
            nym_vpn_api_client,
            min_gateway_performance: config.min_gateway_performance,
        })
    }

    pub fn mixnet_min_performance(&self) -> Option<Percent> {
        self.min_gateway_performance
            .as_ref()
            .and_then(|min_performance| min_performance.mixnet_min_performance)
    }

    pub fn vpn_min_performance(&self) -> Option<Percent> {
        self.min_gateway_performance
            .as_ref()
            .and_then(|min_performance| min_performance.vpn_min_performance)
    }

    async fn lookup_described_nodes(&self) -> Result<Vec<NymNodeDescription>> {
        info!("Fetching all described nodes from nym-api...");
        self.api_client
            .get_all_described_nodes()
            .await
            .map_err(Error::FailedToLookupDescribedGateways)
    }

    async fn lookup_skimmed_gateways(&self) -> Result<Vec<SkimmedNode>> {
        info!("Fetching skimmed entry assigned nodes from nym-api...");
        self.api_client
            .get_all_basic_entry_assigned_nodes()
            .await
            .map_err(Error::FailedToLookupSkimmedGateways)
    }

    async fn lookup_skimmed_nodes(&self) -> Result<Vec<SkimmedNode>> {
        info!("Fetching skimmed entry assigned nodes from nym-api...");
        self.api_client
            .get_all_basic_nodes()
            .await
            .map_err(Error::FailedToLookupSkimmedNodes)
    }

    pub async fn lookup_low_latency_entry_gateway(&self) -> Result<Gateway> {
        debug!("Fetching low latency entry gateway...");
        let gateways = self.lookup_gateways(GatewayType::MixnetEntry).await?;
        gateways.random_low_latency_gateway().await
    }

    pub async fn lookup_gateway_ip_from_nym_api(&self, gateway_identity: &str) -> Result<IpAddr> {
        info!("Fetching gateway ip from nym-api...");
        let mut ips = self
            .api_client
            .get_all_described_nodes()
            .await?
            .iter()
            .find_map(|node| {
                if node
                    .description
                    .host_information
                    .keys
                    .ed25519
                    .to_base58_string()
                    == gateway_identity
                {
                    Some(node.description.host_information.ip_address.clone())
                } else {
                    None
                }
            })
            .ok_or(Error::RequestedGatewayIdNotFound(
                gateway_identity.to_string(),
            ))?;

        if ips.is_empty() {
            // nym-api should forbid this from ever happening, but we don't want to accidentally panic
            // if this assumption fails
            warn!("somehow {gateway_identity} hasn't provided any ip addresses!");
            return Err(Error::RequestedGatewayIdNotFound(
                gateway_identity.to_string(),
            ));
        }

        info!("found the following ips for {gateway_identity}: {ips:?}");
        if ips.len() == 1 {
            // SAFETY: the vector is not empty, so unwrap is fine
            Ok(ips.pop().unwrap())
        } else {
            // chose a random one if there's more than one
            // SAFETY: the vector is not empty, so unwrap is fine
            let mut rng = thread_rng();
            let ip = ips.choose(&mut rng).unwrap();
            Ok(*ip)
        }
    }

    pub async fn lookup_all_gateways_from_nym_api(&self) -> Result<GatewayList> {
        let mut gateways = self
            .lookup_described_nodes()
            .await?
            .into_iter()
            .filter(|node| node.description.declared_role.entry)
            .filter_map(|gw| {
                Gateway::try_from(gw)
                    .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                    .ok()
            })
            .collect::<Vec<_>>();
        let skimmed_gateways = self.lookup_skimmed_gateways().await?;
        append_performance(&mut gateways, skimmed_gateways);
        filter_on_mixnet_min_performance(&mut gateways, &self.min_gateway_performance);
        Ok(GatewayList::new(gateways))
    }

    pub async fn lookup_all_nymnodes(&self) -> Result<NymNodeList> {
        let mut nodes = self
            .lookup_described_nodes()
            .await?
            .into_iter()
            .filter_map(|gw| {
                NymNode::try_from(gw)
                    .inspect_err(|err| error!("Failed to parse node: {err}"))
                    .ok()
            })
            .collect::<Vec<_>>();
        let skimmed_nodes = self.lookup_skimmed_nodes().await?;
        append_performance(&mut nodes, skimmed_nodes);
        filter_on_mixnet_min_performance(&mut nodes, &self.min_gateway_performance);
        Ok(GatewayList::new(nodes))
    }

    pub async fn lookup_gateways_from_nym_api(&self, gw_type: GatewayType) -> Result<GatewayList> {
        match gw_type {
            GatewayType::MixnetEntry => self.lookup_entry_gateways_from_nym_api().await,
            GatewayType::MixnetExit => self.lookup_exit_gateways_from_nym_api().await,
            GatewayType::Wg => self.lookup_vpn_gateways_from_nym_api().await,
        }
    }

    // This is currently the same as the set of all gateways, but it doesn't have to be.
    async fn lookup_entry_gateways_from_nym_api(&self) -> Result<GatewayList> {
        self.lookup_all_gateways_from_nym_api().await
    }

    async fn lookup_exit_gateways_from_nym_api(&self) -> Result<GatewayList> {
        self.lookup_all_gateways_from_nym_api()
            .await
            .map(GatewayList::into_exit_gateways)
    }

    async fn lookup_vpn_gateways_from_nym_api(&self) -> Result<GatewayList> {
        self.lookup_all_gateways_from_nym_api()
            .await
            .map(GatewayList::into_vpn_gateways)
    }

    pub async fn lookup_gateway_ip(&self, gateway_identity: &str) -> Result<IpAddr> {
        if let Some(nym_vpn_api_client) = &self.nym_vpn_api_client {
            info!("Fetching gateway ip from nym-vpn-api...");
            let gateway = nym_vpn_api_client
                .get_gateways(None)
                .await?
                .into_iter()
                .find_map(|gw| {
                    if gw.identity_key != gateway_identity {
                        None
                    } else {
                        Gateway::try_from(gw)
                            .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                            .ok()
                    }
                })
                .ok_or_else(|| Error::RequestedGatewayIdNotFound(gateway_identity.to_string()))?;
            gateway
                .lookup_ip()
                .ok_or(Error::FailedToLookupIp(gateway_identity.to_string()))
        } else {
            warn!("OPERATING IN FALLBACK MODE WITHOUT NYM-VPN-API!");
            self.lookup_gateway_ip_from_nym_api(gateway_identity).await
        }
    }

    pub async fn lookup_all_gateways(&self) -> Result<GatewayList> {
        if let Some(nym_vpn_api_client) = &self.nym_vpn_api_client {
            info!("Fetching all gateways from nym-vpn-api...");
            let gateways: Vec<_> = nym_vpn_api_client
                .get_gateways(self.min_gateway_performance)
                .await?
                .into_iter()
                .filter_map(|gw| {
                    Gateway::try_from(gw)
                        .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                        .ok()
                })
                .collect();
            Ok(GatewayList::new(gateways))
        } else {
            warn!("OPERATING IN FALLBACK MODE WITHOUT NYM-VPN-API!");
            self.lookup_all_gateways_from_nym_api().await
        }
    }

    pub async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList> {
        if let Some(nym_vpn_api_client) = &self.nym_vpn_api_client {
            info!("Fetching gateways from nym-vpn-api...");
            let gateways: Vec<_> = nym_vpn_api_client
                .get_gateways_by_type(gw_type.into(), self.min_gateway_performance)
                .await?
                .into_iter()
                .filter_map(|gw| {
                    Gateway::try_from(gw)
                        .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                        .ok()
                })
                .collect();
            Ok(GatewayList::new(gateways))
        } else {
            warn!("OPERATING IN FALLBACK MODE WITHOUT NYM-VPN-API!");
            self.lookup_gateways_from_nym_api(gw_type).await
        }
    }

    pub async fn lookup_countries(&self, gw_type: GatewayType) -> Result<Vec<Country>> {
        if let Some(nym_vpn_api_client) = &self.nym_vpn_api_client {
            info!("Fetching entry countries from nym-vpn-api...");
            Ok(nym_vpn_api_client
                .get_gateway_countries_by_type(gw_type.into(), self.min_gateway_performance)
                .await?
                .into_iter()
                .map(Country::from)
                .collect())
        } else {
            warn!("OPERATING IN FALLBACK MODE WITHOUT NYM-VPN-API!");
            self.lookup_gateways_from_nym_api(gw_type)
                .await
                .map(GatewayList::into_countries)
        }
    }
}

// Append the performance to the gateways. This is a temporary hack until the nymvpn.com endpoints
// are updated to also include this field.
fn append_performance(
    gateways: &mut [Gateway],
    basic_gw: Vec<nym_validator_client::nym_nodes::SkimmedNode>,
) {
    info!("Appending mixnet_performance to gateways");
    for gateway in gateways.iter_mut() {
        if let Some(basic_gw) = basic_gw
            .iter()
            .find(|bgw| bgw.ed25519_identity_pubkey == gateway.identity())
        {
            gateway.mixnet_performance = Some(basic_gw.performance);
        } else {
            tracing::warn!(
                "Failed to append mixnet_performance, node {} not found among the skimmed nodes",
                gateway.identity()
            );
        }
    }
}

fn filter_on_mixnet_min_performance(
    gateways: &mut Vec<Gateway>,
    min_gateway_performance: &Option<GatewayMinPerformance>,
) {
    if let Some(min_performance) = min_gateway_performance {
        if let Some(mixnet_min_performance) = min_performance.mixnet_min_performance {
            tracing::info!(
                "Filtering gateways based on mixnet_min_performance: {:?}",
                min_performance
            );
            gateways.retain(|gateway| {
                gateway.mixnet_performance.unwrap_or_default() >= mixnet_min_performance
            });
        }
    }
}

#[cfg(test)]
mod test {
    use nym_sdk::UserAgent;

    use super::*;

    fn user_agent() -> UserAgent {
        UserAgent {
            application: "test".to_string(),
            version: "0.0.1".to_string(),
            platform: "test".to_string(),
            git_commit: "test".to_string(),
        }
    }

    // TODO: Remove ignore when magura hits mainnet
    #[ignore]
    #[tokio::test]
    async fn lookup_described_gateways() {
        let config = Config::new_mainnet();
        let client = GatewayClient::new(config, user_agent()).unwrap();
        let gateways = client.lookup_described_nodes().await.unwrap();
        assert!(!gateways.is_empty());
    }

    // TODO: Remove ignore when magura hits mainnet
    #[ignore]
    #[tokio::test]
    async fn lookup_gateways_in_nym_vpn_api() {
        let config = Config::new_mainnet();
        let client = GatewayClient::new(config, user_agent()).unwrap();
        let gateways = client
            .lookup_gateways(GatewayType::MixnetExit)
            .await
            .unwrap();
        assert!(!gateways.is_empty());
    }
}
