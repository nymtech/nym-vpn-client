// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) mod response;

mod account_management;
mod discovery;
mod envs;
mod feature_flags;
mod nym_network;
mod nym_vpn_network;
mod refresh;
mod system_messages;
mod util;

pub use account_management::{AccountManagement, ParsedAccountLinks};
pub use feature_flags::FeatureFlags;
use futures_util::FutureExt;
pub use nym_network::NymNetwork;
pub use nym_vpn_network::NymVpnNetwork;
pub use system_messages::{SystemMessage, SystemMessages};

use discovery::Discovery;
use envs::RegisteredNetworks;
use nym_config::defaults::NymNetworkDetails;
use tokio::join;

use std::{path::Path, time::Duration};

const NETWORKS_SUBDIR: &str = "networks";

// Refresh the discovery and network details files periodically
// const MAX_FILE_AGE: Duration = Duration::from_secs(60 * 60 * 24);
const MAX_FILE_AGE: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub struct Network {
    pub nym_network: NymNetwork,
    pub nym_vpn_network: NymVpnNetwork,
    pub feature_flags: Option<FeatureFlags>,
}

impl Network {
    pub fn nym_network_details(&self) -> &NymNetworkDetails {
        &self.nym_network.network
    }

    pub fn export_to_env(&self) {
        self.nym_network.export_to_env();
        self.nym_vpn_network.export_to_env();
    }

    // Fetch network information directly from the endpoint without going through the path of first
    // persisting to disk etc.
    // Currently used on mobile only.
    pub fn fetch(network_name: &str) -> anyhow::Result<Self> {
        let discovery = Discovery::fetch(network_name)?;
        let feature_flags = discovery.feature_flags.clone();
        let nym_network = discovery.fetch_nym_network_details()?;
        let nym_vpn_network = NymVpnNetwork::from(discovery);

        Ok(Network {
            nym_network,
            nym_vpn_network,
            feature_flags,
        })
    }

    // Query the network name for both urls and check that it matches
    // TODO: integrate with validator-client and/or nym-vpn-api-client
    pub async fn check_consistency(&self) -> anyhow::Result<bool> {
        tracing::info!("Checking network consistency");
        let nym_api_url = self
            .nym_network
            .network
            .endpoints
            .first()
            .and_then(|v| v.api_url())
            .ok_or(anyhow::anyhow!("No endpoints found"))?;
        let network_name = discovery::fetch_nym_network_details(&nym_api_url)
            .map(|resp| resp.map(|d| d.network.network_name));

        let nym_vpn_api_url = self.nym_vpn_network.nym_vpn_api_url.clone();
        let vpn_network_name = discovery::fetch_nym_vpn_network_details(&nym_vpn_api_url)
            .map(|resp| resp.map(|d| d.network_name));

        let (network_name, vpn_network_name) = join!(network_name, vpn_network_name);
        let network_name = network_name?;
        let vpn_network_name = vpn_network_name?;

        tracing::info!("nym network name: {network_name}");
        tracing::info!("nym-vpn network name: {vpn_network_name}");
        Ok(network_name == vpn_network_name)
    }

    pub fn api_url(&self) -> Option<url::Url> {
        self.nym_network_details()
            .endpoints
            .first()
            .and_then(|endpoint| endpoint.api_url())
    }

    pub fn vpn_api_url(&self) -> url::Url {
        self.nym_vpn_network.nym_vpn_api_url.clone()
    }
}

pub fn discover_networks(config_path: &Path) -> anyhow::Result<RegisteredNetworks> {
    RegisteredNetworks::ensure_exists(config_path)
}

pub fn discover_env(config_path: &Path, network_name: &str) -> anyhow::Result<Network> {
    tracing::trace!(
        "Discovering network details: config_path={:?}, network_name={}",
        config_path,
        network_name
    );

    // Lookup network discovery to bootstrap
    let discovery = Discovery::ensure_exists(config_path, network_name)?;

    tracing::info!("Discovery: {:#?}", discovery);

    tracing::info!("System messages: {}", discovery.system_messages);

    let feature_flags = discovery.feature_flags.clone();
    tracing::info!("Feature flags: {:#?}", feature_flags);

    // Using discovery, fetch and setup nym network details
    let nym_network = NymNetwork::ensure_exists(config_path, &discovery)?;

    // Using discovery, setup nym vpn network details
    let nym_vpn_network = NymVpnNetwork::from(discovery);

    Ok(Network {
        nym_network,
        nym_vpn_network,
        feature_flags,
    })
}

pub fn manual_env(network_details: &NymNetworkDetails) -> anyhow::Result<Network> {
    let nym_network = NymNetwork::from(network_details.clone());
    let nym_vpn_network = NymVpnNetwork::try_from(network_details)?;

    Ok(Network {
        nym_network,
        nym_vpn_network,
        feature_flags: None,
    })
}
