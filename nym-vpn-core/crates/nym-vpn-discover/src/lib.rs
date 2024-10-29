// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod bootstrap;
mod nym_network;
mod nym_vpn_network;
mod refresh;
mod util;

pub use nym_network::NymNetwork;
pub use nym_vpn_network::NymVpnNetwork;

use bootstrap::Discovery;
use nym_config::defaults::NymNetworkDetails;

use std::{path::Path, time::Duration};

const NETWORKS_SUBDIR: &str = "networks";

// Refresh the discovery and network details files periodically
const MAX_FILE_AGE: Duration = Duration::from_secs(60 * 60 * 24);

#[derive(Clone, Debug)]
pub struct Network {
    pub nym_network: NymNetwork,
    pub nym_vpn_network: NymVpnNetwork,
}

impl Network {
    pub fn nym_network_details(&self) -> &NymNetworkDetails {
        &self.nym_network.network
    }

    pub fn export_to_env(&self) {
        self.nym_network.export_to_env();
        self.nym_vpn_network.export_to_env();
    }

    pub fn fetch(network_name: &str) -> anyhow::Result<Self> {
        let discovery = Discovery::fetch(network_name)?;
        let nym_network = discovery.fetch_nym_network_details()?;
        let nym_vpn_network = NymVpnNetwork::from(discovery);

        Ok(Network {
            nym_network,
            nym_vpn_network,
        })
    }
}

pub fn discover_env(config_path: &Path, network_name: &str) -> anyhow::Result<Network> {
    // Lookup network discovery to bootstrap
    let discovery = Discovery::ensure_exists(config_path, network_name)?;

    // Using discovery, fetch and setup nym network details
    let nym_network = NymNetwork::ensure_exists(config_path, &discovery)?;

    // Using discovery, setup nym vpn network details
    let nym_vpn_network = NymVpnNetwork::from(discovery);

    Ok(Network {
        nym_network,
        nym_vpn_network,
    })
}

pub fn manual_env(network_details: &NymNetworkDetails) -> anyhow::Result<Network> {
    let nym_network = NymNetwork::from(network_details.clone());
    let nym_vpn_network = NymVpnNetwork::try_from(network_details)?;

    Ok(Network {
        nym_network,
        nym_vpn_network,
    })
}
