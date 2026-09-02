// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_http_api_client::Client as HttpApiClient;
use nym_sdk::{NymNetworkDetails, UserAgent};
use nym_validator_client::nym_api::NymApiClientExt;
use nym_vpn_api_client::{VpnApiClient, api_urls_to_urls, fronted_http_client};

use crate::{Error, Result, discovery::Discovery, envs::RegisteredNetworks};

const NETWORK_TIMEOUT: Duration = Duration::from_secs(10);

/// Network fetcher responsible for interaction with Nym API.
#[derive(Debug)]
pub struct Fetcher {
    api_client: HttpApiClient,
    vpn_api_client: VpnApiClient,
    user_agent: Option<UserAgent>,
    discovery: Box<Discovery>,
}

impl Fetcher {
    /// Create an instance of `Fetcher` using HTTP API endpoints from the given discovery.
    pub async fn new(discovery: Discovery, user_agent: Option<UserAgent>) -> Result<Self> {
        Ok(Self {
            user_agent: user_agent.clone(),
            api_client: build_api_client(&discovery, user_agent.clone()).await?,
            vpn_api_client: build_vpn_api_client(&discovery, user_agent).await?,
            discovery: Box::new(discovery),
        })
    }

    /// Update internal discovery used by the fetcher.
    /// This causes recreation of the underlying HTTP API clients.
    pub(crate) async fn set_discovery(&mut self, new_discovery: Discovery) -> Result<()> {
        if *self.discovery == new_discovery {
            return Ok(());
        }

        // Build both clients before touching any state, so a failure here leaves the fetcher
        // on its previous, known-good discovery and clients rather than a mix of old and new.
        let api_client = build_api_client(&new_discovery, self.user_agent.clone()).await?;
        let vpn_api_client = build_vpn_api_client(&new_discovery, self.user_agent.clone()).await?;

        self.api_client = api_client;
        self.vpn_api_client = vpn_api_client;
        *self.discovery = new_discovery;

        Ok(())
    }

    /// Fetch registered networks from the API.
    pub async fn fetch_registered_networks(&self) -> Result<RegisteredNetworks> {
        self.vpn_api_client
            .get_wellknown_envs()
            .await
            .map_err(Error::GetWellKnownEnvs)
            .map(RegisteredNetworks::new)
            .map(RegisteredNetworks::without_retired)
    }

    /// Fetch discovery information from the API.
    pub async fn fetch_discovery(&self, network_name: &str) -> Result<Discovery> {
        self.vpn_api_client
            .get_wellknown_discovery(network_name)
            .await
            .map_err(Error::GetWellKnownDiscovery)
            .and_then(|response| {
                Discovery::try_from(response).map_err(Error::ConvertWellKnownDiscovery)
            })
    }

    /// Fetch network details from the API.
    pub async fn fetch_network_details(&self) -> Result<Box<NymNetworkDetails>> {
        self.api_client
            .get_network_details()
            .await
            .map(|response| response.network)
            .map(Box::new)
            .map_err(Box::new)
            .map_err(Error::GetNetworkDetails)
    }
}

async fn build_api_client(
    discovery: &Discovery,
    user_agent: Option<UserAgent>,
) -> Result<HttpApiClient> {
    let api_urls =
        api_urls_to_urls(&discovery.nym_api_urls()).map_err(Error::CreateVpnApiClient)?;

    fronted_http_client::fronted_http_client(api_urls, user_agent, Some(NETWORK_TIMEOUT))
        .await
        .map_err(Error::CreateVpnApiClient)
}

async fn build_vpn_api_client(
    discovery: &Discovery,
    user_agent: Option<UserAgent>,
) -> Result<VpnApiClient> {
    let vpn_api_urls =
        api_urls_to_urls(&discovery.nym_vpn_api_urls()).map_err(Error::CreateVpnApiClient)?;

    VpnApiClient::new(vpn_api_urls, user_agent)
        .await
        .map_err(Error::CreateVpnApiClient)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_fetch() {
        let network_name = "mainnet";
        let fetcher = Fetcher::new(Discovery::default_mainnet(), None)
            .await
            .unwrap();
        let discovery = fetcher.fetch_discovery(network_name).await.unwrap();
        assert_eq!(discovery.network_name, network_name);
    }
}
