// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_http_api_client::Client as HttpApiClient;
use nym_network_defaults::{ApiUrl, v2::NymNetworkDetails};
use nym_sdk::UserAgent;
use nym_validator_client::nym_api::NymApiClientExt;
use nym_vpn_api_client::{VpnApiClient, api_urls_to_urls, fronted_http_client};

use crate::{Error, Result, discovery::Discovery, envs::RegisteredNetworks};

const NETWORK_TIMEOUT: Duration = Duration::from_secs(10);

/// Network fetcher responsible for interaction with Nym API.
#[derive(Debug)]
pub struct Fetcher {
    // One client per known host, kept so each remembers its failed routes between refreshes.
    api_clients: Vec<HttpApiClient>,
    discovery_clients: Vec<VpnApiClient>,
    user_agent: Option<UserAgent>,
    discovery: Box<Discovery>,
}

impl Fetcher {
    /// Create an instance of `Fetcher` using HTTP API endpoints from the given discovery.
    pub fn new(discovery: Discovery, user_agent: Option<UserAgent>) -> Result<Self> {
        Ok(Self {
            user_agent: user_agent.clone(),
            api_clients: build_api_clients(&discovery, user_agent.clone())?,
            discovery_clients: build_discovery_clients(&discovery, user_agent)?,
            discovery: Box::new(discovery),
        })
    }

    /// Update internal discovery used by the fetcher.
    /// This causes recreation of the underlying HTTP API clients.
    pub(crate) fn set_discovery(&mut self, new_discovery: Discovery) -> Result<()> {
        if *self.discovery == new_discovery {
            return Ok(());
        }

        self.api_clients = build_api_clients(&new_discovery, self.user_agent.clone())?;
        self.discovery_clients = build_discovery_clients(&new_discovery, self.user_agent.clone())?;
        *self.discovery = new_discovery;

        Ok(())
    }

    /// Fetch registered networks from the API.
    pub async fn fetch_registered_networks(&self) -> Result<RegisteredNetworks> {
        try_each(&self.discovery_clients, |client| async move {
            client
                .get_wellknown_envs()
                .await
                .map_err(Error::GetWellKnownEnvs)
        })
        .await
        .map(RegisteredNetworks::new)
        .map(RegisteredNetworks::without_retired)
    }

    /// Fetch discovery information from the API.
    pub async fn fetch_discovery(&self, network_name: &str) -> Result<Discovery> {
        try_each(&self.discovery_clients, |client| {
            self.fetch_discovery_from(client, network_name)
        })
        .await
    }

    async fn fetch_discovery_from(
        &self,
        client: VpnApiClient,
        network_name: &str,
    ) -> Result<Discovery> {
        let response = client
            .get_wellknown_discovery(network_name)
            .await
            .map_err(Error::GetWellKnownDiscovery)?;
        Discovery::try_from(response)
    }

    /// Fetch network details from the API.
    pub async fn fetch_network_details(&self) -> Result<Box<NymNetworkDetails>> {
        try_each(&self.api_clients, |client| async move {
            client
                .get_network_details_v2()
                .await
                .map(|response| response.network)
                .map(Box::new)
                .map_err(Box::new)
                .map_err(Error::GetNetworkDetails)
        })
        .await
    }
}

/// Tries each known host in turn and returns the first answer that works.
async fn try_each<C: Clone, T, Fut>(clients: &[C], fetch: impl Fn(C) -> Fut) -> Result<T>
where
    Fut: Future<Output = Result<T>>,
{
    let mut last_error = Error::InvalidDiscoveryNetworking;
    for client in clients {
        match fetch(client.clone()).await {
            Ok(value) => return Ok(value),
            Err(err) => {
                tracing::debug!("one host failed, trying the next: {err}");
                last_error = err;
            }
        }
    }
    Err(last_error)
}

/// A persisted or fetched discovery can be stale or wrong, so the bundled URLs stay behind it.
fn with_bundled_urls(discovery: &Discovery, urls: fn(&Discovery) -> Vec<ApiUrl>) -> Vec<ApiUrl> {
    let bundled = Discovery::default_discovery(&discovery.network_name)
        .map(|bundled| urls(&bundled))
        .unwrap_or_default();
    crate::discovery_first(urls(discovery), bundled)
}

fn build_discovery_clients(
    discovery: &Discovery,
    user_agent: Option<UserAgent>,
) -> Result<Vec<VpnApiClient>> {
    with_bundled_urls(discovery, Discovery::nym_vpn_api_urls)
        .into_iter()
        .map(|url| {
            let urls = api_urls_to_urls(&[url]).map_err(Error::CreateVpnApiClient)?;
            VpnApiClient::new(urls, user_agent.clone()).map_err(Error::CreateVpnApiClient)
        })
        .collect()
}

fn build_api_clients(
    discovery: &Discovery,
    user_agent: Option<UserAgent>,
) -> Result<Vec<HttpApiClient>> {
    with_bundled_urls(discovery, Discovery::nym_api_urls)
        .into_iter()
        .map(|url| {
            let urls = api_urls_to_urls(&[url]).map_err(Error::CreateVpnApiClient)?;
            fronted_http_client::fronted_http_client(
                urls,
                user_agent.clone(),
                Some(NETWORK_TIMEOUT),
            )
            .map_err(Error::CreateVpnApiClient)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetcher_urls_keep_the_bundled_ones_behind_a_changed_discovery() {
        let mut discovery = Discovery::default_mainnet();
        let stale = ApiUrl {
            url: "https://other.test/api/".to_owned(),
            front_hosts: None,
        };
        discovery.networking.nym_vpn_api_urls = vec![stale.clone()];

        let urls = with_bundled_urls(&discovery, Discovery::nym_vpn_api_urls);

        assert_eq!(urls[0], stale);
        assert_eq!(urls[1..], Discovery::default_mainnet().nym_vpn_api_urls());
    }

    #[tokio::test]
    async fn test_discovery_fetch() {
        let network_name = "mainnet";
        let fetcher = Fetcher::new(Discovery::default_mainnet(), None).unwrap();
        let discovery = fetcher.fetch_discovery(network_name).await.unwrap();
        assert_eq!(discovery.network_name, network_name);
    }
}
