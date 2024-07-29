// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    entries::gateway::{Gateway, GatewayList},
    error::Result,
    helpers::{select_random_low_latency_described_gateway, try_resolve_hostname},
    AuthAddress, Error, IpPacketRouterAddress,
};
use nym_sdk::{mixnet::Recipient, UserAgent};
use nym_topology::IntoGatewayNode;
use nym_validator_client::{models::DescribedGateway, NymApiClient};
use nym_vpn_api_client::VpnApiClientExt;
use std::{fmt, net::IpAddr, time::Duration};
use tracing::{debug, error, info};
use url::Url;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_url: Url,
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

        let default_nym_vpn_api_url = mainnet_network_defaults
            .nym_vpn_api_url()
            .expect("rust sdk mainnet default nym-vpn-api url not parseable");

        Config {
            api_url: default_api_url,
            nym_vpn_api_url: Some(default_nym_vpn_api_url),
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

        // The vpn api url is strictly not needed, so skip the expect here
        let nym_vpn_api_url = network.nym_vpn_api_url();

        Config {
            api_url,
            nym_vpn_api_url,
        }
    }

    pub fn new_from_urls(api_url: Url, nym_vpn_api_url: Option<Url>) -> Self {
        Config {
            api_url,
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
    nym_vpn_api_client: Option<nym_vpn_api_client::Client>,
}

impl GatewayClient {
    pub fn new(config: Config, user_agent: UserAgent) -> Result<Self> {
        let api_client = NymApiClient::new_with_user_agent(config.api_url, user_agent.clone());
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
            nym_vpn_api_client,
        })
    }

    async fn lookup_described_gateways(&self) -> Result<Vec<DescribedGateway>> {
        info!("Fetching gateways from nym-api...");
        self.api_client
            .get_cached_described_gateways()
            .await
            .map_err(|source| Error::FailedToLookupDescribedGateways { source })
    }

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
                    .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                    .ok()
            })
            .collect();
        Ok(GatewayList::new(gateways))
    }

    pub async fn lookup_entry_gateways(&self) -> Result<GatewayList> {
        if let Some(nym_vpn_api_client) = &self.nym_vpn_api_client {
            info!("Fetching entry gateways from nym-vpn-api...");
            let mut entry_gateways: Vec<_> = nym_vpn_api_client
                .get_entry_gateways()
                .await?
                .into_iter()
                .filter_map(|gw| {
                    Gateway::try_from(gw)
                        .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                        .ok()
                })
                .collect();

            // Lookup the IPR and authenticator addresses from the nym-api as a temporary hack until
            // the nymvpn.com endpoints are updated to also include these fields.
            let described_gateways = self.lookup_described_gateways().await?;
            append_ipr_and_authenticator_addresses(&mut entry_gateways, described_gateways);
            Ok(GatewayList::new(entry_gateways))
        } else {
            self.lookup_all_gateways_from_nym_api().await
        }
    }

    pub async fn lookup_exit_gateways(&self) -> Result<GatewayList> {
        if let Some(nym_vpn_api_client) = &self.nym_vpn_api_client {
            info!("Fetching exit gateways from nym-vpn-api...");
            let mut exit_gateways: Vec<_> = nym_vpn_api_client
                .get_exit_gateways()
                .await?
                .into_iter()
                .filter_map(|gw| {
                    Gateway::try_from(gw)
                        .inspect_err(|err| error!("Failed to parse gateway: {err}"))
                        .ok()
                })
                .collect();

            // Lookup the IPR and authenticator addresses from the nym-api as a temporary hack until
            // the nymvpn.com endpoints are updated to also include these fields.
            let described_gateways = self.lookup_described_gateways().await?;
            append_ipr_and_authenticator_addresses(&mut exit_gateways, described_gateways);
            Ok(GatewayList::new(exit_gateways))
        } else {
            self.lookup_all_gateways_from_nym_api()
                .await
                .map(|gateways| gateways.into_exit_gateways())
        }
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
