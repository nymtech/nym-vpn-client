// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod bootstrap;
mod global_config;
mod nym_network;
mod nym_vpn_network;
mod refresh;

pub(crate) use global_config::GlobalConfigFile;

use std::time::Duration;

const NETWORKS_SUBDIR: &str = "networks";

// Refresh the discovery and network details files periodically
const MAX_FILE_AGE: Duration = Duration::from_secs(60 * 60 * 24);

#[derive(Clone, Debug)]
pub(crate) struct Network {
    #[allow(unused)]
    pub(crate) nym_network: nym_network::NymNetwork,
    #[allow(unused)]
    pub(crate) nym_vpn_network: nym_vpn_network::NymVpnNetwork,
}

pub(crate) fn discover_env(network_name: &str) -> anyhow::Result<Network> {
    // Lookup network discovery to bootstrap
    let discovery = bootstrap::Discovery::ensure_exists(network_name)?;

    // Using discovery, fetch and setup nym network details
    let nym_network = nym_network::NymNetwork::ensure_exists(&discovery)?;
    nym_network.export_to_env();
    crate::set_global_network_details(nym_network.network.clone())?;

    // Using discovery, setup nym vpn network details
    let nym_vpn_network = nym_vpn_network::NymVpnNetwork::from(discovery);
    nym_vpn_network.export_to_env();

    Ok(Network {
        nym_network,
        nym_vpn_network,
    })
}

pub(crate) fn manual_env(
    network_details: &nym_vpn_lib::nym_config::defaults::NymNetworkDetails,
) -> anyhow::Result<Network> {
    let nym_network = nym_network::NymNetwork::from(network_details.clone());
    nym_network.export_to_env();
    crate::set_global_network_details(network_details.clone())?;

    let nym_vpn_network = nym_vpn_network::NymVpnNetwork::try_from(network_details)?;
    nym_vpn_network.export_to_env();

    Ok(Network {
        nym_network,
        nym_vpn_network,
    })
}
