// Copyright 2023-2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::{HashMap, HashSet},
    fmt,
    net::{IpAddr, SocketAddr},
};

use nym_network_defaults::ApiUrl;
use nym_sdk::UserAgent;
use nym_validator_client::{
    models::described::v2::NymNodeDescriptionV2, nym_api::NymApiClientExt,
    nym_nodes::SkimmedNodesWithMetadata,
};
use nym_vpn_api_client::{
    ResolverOverrides, api_urls_to_urls, fronted_http_client,
    types::{GatewayMinPerformance, Percent},
    url_to_socket_addr,
};
use rand::{prelude::SliceRandom, thread_rng};
use tracing::{debug, error, warn};
use url::Url;

use crate::{
    Error, NymNode,
    entries::gateway::{Gateway, GatewayList, GatewayType, NymNodeList},
    error::Result,
};

#[derive(Clone, Debug)]
pub struct Config {
    nyxd_url: Url,
    nym_api_urls: Vec<ApiUrl>,
    nym_vpn_api_urls: Vec<ApiUrl>,
    min_gateway_performance: Option<GatewayMinPerformance>,
}

impl Config {
    pub fn new(
        nyxd_url: Url,
        nym_api_urls: Vec<ApiUrl>,
        nym_vpn_api_urls: Vec<ApiUrl>,
        min_gateway_performance: Option<GatewayMinPerformance>,
    ) -> Result<Self> {
        if nym_api_urls.is_empty() {
            return Err(Error::InvalidConfiguration(
                "NYM API URLs cannot be empty".to_string(),
            ));
        }

        if nym_vpn_api_urls.is_empty() {
            return Err(Error::InvalidConfiguration(
                "NYM VPN API URLs cannot be empty".to_string(),
            ));
        }

        Ok(Config {
            nyxd_url,
            nym_api_urls,
            nym_vpn_api_urls,
            min_gateway_performance,
        })
    }

    pub fn with_min_gateway_performance(
        mut self,
        min_gateway_performance: GatewayMinPerformance,
    ) -> Self {
        self.min_gateway_performance = Some(min_gateway_performance);
        self
    }

    pub fn nyxd_url(&self) -> &Url {
        &self.nyxd_url
    }

    pub fn nym_api_urls(&self) -> &[ApiUrl] {
        &self.nym_api_urls
    }

    pub fn nym_vpn_api_urls(&self) -> &[ApiUrl] {
        &self.nym_vpn_api_urls
    }

    pub fn min_gateway_performance(&self) -> Option<GatewayMinPerformance> {
        self.min_gateway_performance
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "nyxd_url: {}, nym_api_urls: {}, nym_vpn_api_urls: {}, min_gateway_performance: {:?}",
            self.nyxd_url,
            self.nym_api_urls
                .iter()
                .map(|url| url.url.clone())
                .collect::<Vec<_>>()
                .join(", "),
            self.nym_vpn_api_urls
                .iter()
                .map(|url| url.url.clone())
                .collect::<Vec<_>>()
                .join(", "),
            self.min_gateway_performance,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub nyxd_url: Url,
    pub nyxd_socket_addrs: Vec<SocketAddr>,
    pub nym_api_resolver_overrides: ResolverOverrides,
    pub nym_vpn_api_resolver_overrides: ResolverOverrides,
}

impl ResolvedConfig {
    pub async fn from_config(config: &Config) -> Result<Self> {
        let nyxd_url = config.nyxd_url().clone();
        let nyxd_socket_addrs = url_to_socket_addr(&nyxd_url, Some((1, 1))).await?;

        let nym_api_resolver_overrides =
            ResolverOverrides::from_api_urls(&config.nym_api_urls).await?;

        let nym_vpn_api_resolver_overrides =
            ResolverOverrides::from_api_urls(&config.nym_vpn_api_urls).await?;

        Ok(ResolvedConfig {
            nyxd_url,
            nyxd_socket_addrs,
            nym_api_resolver_overrides,
            nym_vpn_api_resolver_overrides,
        })
    }

    pub fn has_resolver_overrides(&self) -> bool {
        !self.nym_api_resolver_overrides.is_empty()
            || !self.nym_vpn_api_resolver_overrides.is_empty()
    }

    pub fn addr_map(&self) -> HashMap<String, Vec<IpAddr>> {
        let mut addr_map = self.nym_vpn_api_resolver_overrides.addr_map();
        for (k, v) in self.nym_api_resolver_overrides.addr_map().into_iter() {
            _ = addr_map.insert(k, v);
        }
        let nyxd_ips: Vec<IpAddr> = self.nyxd_socket_addrs.iter().map(|sa| sa.ip()).collect();

        _ = self
            .nyxd_url
            .host_str()
            .map(|nyxd_domain| addr_map.insert(nyxd_domain.to_string(), nyxd_ips));
        addr_map
    }

    pub fn all_socket_addrs(&self) -> Vec<SocketAddr> {
        let mut socket_addrs = vec![];
        socket_addrs.extend(self.nyxd_socket_addrs.iter());
        socket_addrs.extend(self.nym_api_resolver_overrides.all_addresses());
        socket_addrs.extend(self.nym_vpn_api_resolver_overrides.all_addresses());
        socket_addrs
    }
}

#[derive(Clone)]
pub struct GatewayClient {
    api_client: nym_http_api_client::Client,
    vpn_api_client: nym_vpn_api_client::VpnApiClient,

    #[allow(unused)]
    nyxd_url: Url,

    min_gateway_performance: Option<GatewayMinPerformance>,
    config: Config,
}

impl GatewayClient {
    pub async fn new(config: Config, user_agent: UserAgent) -> Result<Self> {
        let nym_urls = api_urls_to_urls(config.nym_api_urls())?;
        let nym_urls = ResolverOverrides::resolve_and_prune(&nym_urls).await;

        let api_client = fronted_http_client(nym_urls, Some(user_agent.clone()), None)
            .map_err(Error::VpnApiClientError)?;

        let nym_vpn_urls = api_urls_to_urls(config.nym_vpn_api_urls())?;
        let nym_vpn_urls = ResolverOverrides::resolve_and_prune(&nym_vpn_urls).await;

        let vpn_api_client =
            nym_vpn_api_client::VpnApiClient::new(nym_vpn_urls, Some(user_agent.clone()))
                .map_err(Error::VpnApiClientError)?;

        Ok(GatewayClient {
            api_client,
            vpn_api_client,
            nyxd_url: config.nyxd_url.clone(),
            min_gateway_performance: config.min_gateway_performance,
            config,
        })
    }

    // So this function is pretty much the same as new_with_resolver_overrides(),
    // except it uses the network_details.nym_vpn_api_urls to create the vpn_api_client.
    pub async fn from_network(
        config: Config,
        network_details: &nym_network_defaults::NymNetworkDetails,
        user_agent: UserAgent,
    ) -> Result<Self> {
        let nym_urls = api_urls_to_urls(&config.nym_api_urls)?;
        let nym_urls = ResolverOverrides::resolve_and_prune(&nym_urls).await;

        let api_client = fronted_http_client(nym_urls, Some(user_agent.clone()), None)
            .map_err(Error::VpnApiClientError)?;

        let vpn_api_client = nym_vpn_api_client::VpnApiClient::from_network(
            network_details,
            Some(user_agent.clone()),
        )
        .await?;

        Ok(GatewayClient {
            api_client,
            vpn_api_client,
            nyxd_url: config.nyxd_url.clone(),
            min_gateway_performance: config.min_gateway_performance,
            config,
        })
    }

    /// Return the config of this instance.
    pub fn get_config(&self) -> Config {
        self.config.clone()
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

    async fn lookup_described_nodes(&self) -> Result<Vec<NymNodeDescriptionV2>> {
        debug!("Fetching all described nodes from nym-api...");
        self.api_client
            .get_all_described_nodes_v2()
            .await
            .map_err(|e| Error::NymApi {
                source: Box::new(e),
            })
    }

    async fn lookup_skimmed_gateways(&self) -> Result<SkimmedNodesWithMetadata> {
        debug!("Fetching skimmed entry assigned nodes from nym-api...");
        self.api_client
            .get_all_basic_entry_assigned_nodes_with_metadata()
            .await
            .map_err(|e| Error::NymApi {
                source: Box::new(e),
            })
    }

    async fn lookup_skimmed_nodes(&self) -> Result<SkimmedNodesWithMetadata> {
        debug!("Fetching skimmed entry assigned nodes from nym-api...");
        self.api_client
            .get_all_basic_nodes_with_metadata()
            .await
            .map_err(|e| Error::NymApi {
                source: Box::new(e),
            })
    }

    pub async fn lookup_gateway_ip_from_nym_api(&self, gateway_identity: &str) -> Result<IpAddr> {
        debug!("Fetching gateway ip from nym-api...");
        let mut ips = self
            .api_client
            .get_all_described_nodes_v2()
            .await
            .map_err(|e| Error::NymApi {
                source: Box::new(e),
            })?
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

        debug!("found the following ips for {gateway_identity}: {ips:?}");
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
        let skimmed_gateways = self.lookup_skimmed_gateways().await?;
        let key_rotation_id = skimmed_gateways.metadata.rotation_id;

        let mut gateways = self
            .lookup_described_nodes()
            .await?
            .into_iter()
            .filter(|node| node.description.declared_role.entry)
            .filter_map(|gw| {
                Gateway::try_from_node_description(gw, key_rotation_id)
                    .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                    .ok()
            })
            .collect::<Vec<_>>();
        append_performance(&mut gateways, skimmed_gateways.nodes);
        filter_on_mixnet_min_performance(&mut gateways, &self.min_gateway_performance);
        Ok(GatewayList::new(None, gateways))
    }

    pub async fn lookup_all_nymnodes(&self) -> Result<NymNodeList> {
        let skimmed_nodes = self.lookup_skimmed_nodes().await?;
        let key_rotation_id = skimmed_nodes.metadata.rotation_id;

        let mut nodes = self
            .lookup_described_nodes()
            .await?
            .into_iter()
            .filter_map(|gw| {
                NymNode::try_from_node_description(gw, key_rotation_id)
                    .inspect_err(|err| error!("Failed to parse node: {err}"))
                    .ok()
            })
            .collect::<Vec<_>>();
        append_performance(&mut nodes, skimmed_nodes.nodes);
        filter_on_mixnet_min_performance(&mut nodes, &self.min_gateway_performance);
        Ok(GatewayList::new(None, nodes))
    }

    /// Lookup NymNodes with SOCKS5 probe data from VPN API
    /// This is specifically for SOCKS5 Network Requester selection and includes:
    /// - Probe data with SOCKS5 scores from VPN API
    /// - Network Requester addresses from node descriptions
    pub async fn lookup_nymnodes_for_socks5(&self) -> Result<NymNodeList> {
        debug!("Fetching NymNodes from nym-vpn-api for SOCKS5 selection (with probe data)...");
        let vpn_gateways = self
            .vpn_api_client
            .get_gateways(self.min_gateway_performance)
            .await?
            .into_inner();

        // Only fetch NR addresses for the VPN gateways we actually need
        // This avoids building a large HashMap from all described nodes
        let gateway_identities: HashSet<String> = vpn_gateways
            .iter()
            .map(|gw| gw.identity_key.clone())
            .collect();

        let nr_address_map: HashMap<String, String> = self
            .lookup_described_nodes()
            .await?
            .into_iter()
            .filter_map(|node| {
                let identity = node
                    .description
                    .host_information
                    .keys
                    .ed25519
                    .to_base58_string();

                // Only process nodes that are in our VPN gateways list
                if !gateway_identities.contains(&identity) {
                    return None;
                }

                node.description
                    .network_requester
                    .as_ref()
                    .map(|nr| (identity, nr.address.clone()))
            })
            .collect();

        let mut nodes: Vec<_> = vpn_gateways
            .into_iter()
            .filter_map(|gw| {
                let mut gateway = Gateway::try_from(gw)
                    .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                    .ok()?;

                let identity_str = gateway.identity().to_string();
                if let Some(nr_address) = nr_address_map.get(&identity_str) {
                    gateway.nr_address = Some(nr_address.clone());
                }

                Some(gateway)
            })
            .collect();

        // Filter for mixnet performance if configured
        filter_on_mixnet_min_performance(&mut nodes, &self.min_gateway_performance);
        Ok(GatewayList::new(None, nodes))
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
        debug!("Fetching gateway ip from nym-vpn-api...");
        let gateway = self
            .vpn_api_client
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
    }

    pub async fn lookup_all_gateways(&self) -> Result<GatewayList> {
        debug!("Fetching all gateways from nym-vpn-api...");
        let gateways: Vec<_> = self
            .vpn_api_client
            .get_gateways(self.min_gateway_performance)
            .await?
            .into_iter()
            .filter_map(|gw| {
                Gateway::try_from(gw)
                    .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                    .ok()
            })
            .collect();
        Ok(GatewayList::new(None, gateways))
    }

    pub async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList> {
        debug!("Fetching {gw_type} gateways from nym-vpn-api...");
        let raw_gateways = self
            .vpn_api_client
            .get_gateways_by_type(gw_type.into(), self.min_gateway_performance)
            .await?;

        let raw_gateways_vec = raw_gateways.into_inner();

        tracing::debug!(
            "VPN-API returned {} raw gateways for {:?}",
            raw_gateways_vec.len(),
            gw_type
        );

        let gateways: Vec<_> = raw_gateways_vec
            .into_iter()
            .filter_map(|gw| {
                Gateway::try_from(gw)
                    .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                    .ok()
            })
            // we need to filter for possible mixnet blacklisting, as the mixnet channel is a prerequisite for VPN connection too
            .filter(|gw| gw.not_mixnet_blacklisted())
            .collect();

        tracing::debug!(
            "Successfully parsed {} gateways for {:?}",
            gateways.len(),
            gw_type
        );

        Ok(GatewayList::new(Some(gw_type), gateways))
    }
}

// Append the performance to the gateways. This is a temporary hack until the nymvpn.com endpoints
// are updated to also include this field.
fn append_performance(
    gateways: &mut [Gateway],
    basic_gw: Vec<nym_validator_client::nym_nodes::SkimmedNodeV1>,
) {
    debug!("Appending mixnet_performance to gateways");
    for gateway in gateways.iter_mut() {
        if let Some(basic_gw) = basic_gw
            .iter()
            .find(|bgw| bgw.ed25519_identity_pubkey == gateway.identity())
        {
            gateway.mixnet_performance = Some(basic_gw.performance);
        } else {
            warn!(
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
    if let Some(min_performance) = min_gateway_performance
        && let Some(mixnet_min_performance) = min_performance.mixnet_min_performance
    {
        debug!(
            "Filtering gateways based on mixnet_min_performance: {:?}",
            min_performance
        );
        gateways.retain(|gateway| {
            gateway.mixnet_performance.unwrap_or_default() >= mixnet_min_performance
        });
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

    fn new_mainnet() -> Config {
        let mainnet_network_defaults = nym_sdk::NymNetworkDetails::default();
        let default_nyxd_url = mainnet_network_defaults
            .endpoints
            .first()
            .expect("rust sdk mainnet default incorrectly configured")
            .nyxd_url();
        let default_api_urls = mainnet_network_defaults
            .nym_api_urls
            .as_ref()
            .expect("rust sdk mainnet default incorrectly configured")
            .clone();
        let default_nym_vpn_api_urls = mainnet_network_defaults
            .nym_vpn_api_urls
            .as_ref()
            .expect("rust sdk mainnet default incorrectly configured")
            .clone();
        Config {
            nyxd_url: default_nyxd_url,
            nym_api_urls: default_api_urls,
            nym_vpn_api_urls: default_nym_vpn_api_urls,
            min_gateway_performance: None,
        }
    }

    async fn mainnet_gateway_client() -> GatewayClient {
        let config = new_mainnet();
        GatewayClient::new(config, user_agent()).await.unwrap()
    }

    #[tokio::test]
    async fn lookup_described_gateways() {
        let client = mainnet_gateway_client().await;
        let gateways = client.lookup_described_nodes().await.unwrap();
        assert!(!gateways.is_empty());
    }

    #[tokio::test]
    async fn lookup_gateways_in_nym_vpn_api() {
        let client = mainnet_gateway_client().await;
        let gateways = client
            .lookup_gateways(GatewayType::MixnetExit)
            .await
            .unwrap();
        assert!(!gateways.is_empty());
    }
}
