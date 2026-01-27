// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_http_api_client::Client as HttpApiClient;
use nym_sdk::{NymNetworkDetails, UserAgent};
use nym_validator_client::nym_api::NymApiClientExt;
use nym_vpn_api_client::{ResolverOverrides, VpnApiClient, api_urls_to_urls, fronted_http_client};

use crate::{Error, Result, discovery::Discovery, envs::RegisteredNetworks};

const NETWORK_TIMEOUT: Duration = Duration::from_secs(10);

/// Network fetcher responsible for interaction with Nym API.
#[derive(Debug)]
pub struct Fetcher {
    api_client: HttpApiClient,
    vpn_api_client: VpnApiClient,
    user_agent: Option<UserAgent>,
    discovery: Box<Discovery>,
    resolver_overrides: Option<ResolverOverrides>,
}

impl Fetcher {
    /// Create an instance of `Fetcher` using HTTP API endpoints from the given discovery.
    pub fn new(
        discovery: Discovery,
        user_agent: Option<UserAgent>,
        resolver_overrides: Option<&ResolverOverrides>,
    ) -> Result<Self> {
        Ok(Self {
            user_agent: user_agent.clone(),
            resolver_overrides: resolver_overrides.cloned(),
            api_client: build_api_client(&discovery, user_agent.clone(), resolver_overrides)?,
            vpn_api_client: build_vpn_api_client(&discovery, user_agent, resolver_overrides)?,
            discovery: Box::new(discovery),
        })
    }

    /// Update internal discovery used by the fetcher.
    /// This causes recreation of the underlying HTTP API clients.
    pub(crate) fn set_discovery(&mut self, new_discovery: Discovery) -> Result<()> {
        if *self.discovery == new_discovery {
            return Ok(());
        }

        self.api_client = build_api_client(
            &new_discovery,
            self.user_agent.clone(),
            self.resolver_overrides.as_ref(),
        )?;
        self.vpn_api_client = build_vpn_api_client(
            &new_discovery,
            self.user_agent.clone(),
            self.resolver_overrides.as_ref(),
        )?;
        *self.discovery = new_discovery;

        Ok(())
    }

    /// Update resolver overrides used by the fetcher.
    /// This causes recreation of the underlying HTTP API clients.
    pub fn set_resolver_overrides(
        &mut self,
        new_overrides: Option<ResolverOverrides>,
    ) -> Result<bool> {
        if self.resolver_overrides == new_overrides {
            return Ok(false);
        }

        self.api_client = build_api_client(
            &self.discovery,
            self.user_agent.clone(),
            new_overrides.as_ref(),
        )?;

        self.vpn_api_client
            .override_resolver(new_overrides.as_ref())
            .map_err(Error::SetResolverOverrides)?;
        self.resolver_overrides = new_overrides;

        Ok(true)
    }

    /// Fetch registered networks from the API.
    pub async fn fetch_registered_networks(&self) -> Result<RegisteredNetworks> {
        self.vpn_api_client
            .get_wellknown_envs()
            .await
            .map_err(Error::GetWellKnownEnvs)
            .map(RegisteredNetworks::new)
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

fn build_api_client(
    discovery: &Discovery,
    user_agent: Option<UserAgent>,
    resolver_overrides: Option<&ResolverOverrides>,
) -> Result<HttpApiClient> {
    let api_urls =
        api_urls_to_urls(&discovery.nym_api_urls()).map_err(Error::CreateVpnApiClient)?;

    fronted_http_client::fronted_http_client(
        api_urls,
        user_agent,
        Some(NETWORK_TIMEOUT),
        resolver_overrides,
    )
    .map_err(Error::CreateVpnApiClient)
}

fn build_vpn_api_client(
    discovery: &Discovery,
    user_agent: Option<UserAgent>,
    resolver_overrides: Option<&ResolverOverrides>,
) -> Result<VpnApiClient> {
    let vpn_api_urls =
        api_urls_to_urls(&discovery.nym_vpn_api_urls()).map_err(Error::CreateVpnApiClient)?;

    VpnApiClient::new(vpn_api_urls, user_agent, resolver_overrides)
        .map_err(Error::CreateVpnApiClient)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_fetch() {
        let network_name = "mainnet";
        let fetcher = Fetcher::new(Discovery::default_mainnet(), None, None).unwrap();
        let discovery = fetcher.fetch_discovery(network_name).await.unwrap();
        assert_eq!(discovery.network_name, network_name);
    }
}
