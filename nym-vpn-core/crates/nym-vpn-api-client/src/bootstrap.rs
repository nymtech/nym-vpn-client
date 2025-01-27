// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::NO_PARAMS;

use url::Url;

use crate::{
    client::NYM_VPN_API_TIMEOUT,
    error::{Result, VpnApiClientError},
    response::{DiscoveryResponse, RegisteredNetworksResponse},
    routes,
};

/// Bootstrapping Environments and Network Discovery
pub struct BootstrapVpnApiClient {
    inner: nym_http_api_client::Client,
}

impl BootstrapVpnApiClient {
    /// Hard coded well known URL for bootstrapping environment and discovery config
    /// allowing more refined URL usage.
    // hard coded for now.
    const WELLKNOWN_URL: &str = "https://nymvpn.com/api";

    /// Returns a VpnApiClient Based on locally set well known url and empty user agent.
    ///
    /// THIS SHOULD ONLY BE USED FOR BOOTSTRAPPING.
    pub fn new(base_url: Option<Url>) -> Result<Self> {
        let url: Url = base_url.unwrap_or(Self::WELLKNOWN_URL.parse().unwrap());

        nym_http_api_client::Client::builder(url)
            .map(|builder| builder.with_timeout(NYM_VPN_API_TIMEOUT))
            .and_then(|builder| builder.build())
            .map(|c| Self { inner: c })
            .map_err(VpnApiClientError::FailedToCreateVpnApiClient)
    }

    pub async fn get_network_envs(&self) -> Result<RegisteredNetworksResponse> {
        self.inner
            .get_json(
                &[
                    routes::PUBLIC,
                    routes::V1,
                    routes::WELLKNOWN,
                    routes::ENVS_FILE,
                ],
                NO_PARAMS,
            )
            .await
            .map_err(VpnApiClientError::FailedToGetNetworkEnvs)
    }

    pub async fn get_discovery_init(&self, network_name: &str) -> Result<DiscoveryResponse> {
        self.inner
            .get_json(
                &[
                    routes::PUBLIC,
                    routes::V1,
                    routes::WELLKNOWN,
                    network_name,
                    routes::DISCOVERY_FILE,
                ],
                NO_PARAMS,
            )
            .await
            .map_err(VpnApiClientError::FailedToGetDiscoveryInfo)
    }
}
