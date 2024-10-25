// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod bootstrap;
mod global_config;
mod nym_network;
mod nym_vpn_network;
mod refresh;

pub(crate) use global_config::{read_global_config_file, write_global_config_file};

const NETWORKS_SUBDIR: &str = "networks";

pub(crate) fn discover_env(network_name: &str) -> anyhow::Result<()> {
    // Lookup network discovery to bootstrap
    bootstrap::download_or_refresh_discovery_file(network_name)?;
    let discovery = bootstrap::read_discovery_file(network_name)?;

    // Using discovery, fetch and setup nym network details
    if !nym_network::check_if_nym_network_details_file_exists(&discovery.network_name) {
        let network_details =
            nym_network::fetch_nym_network_details(discovery.nym_api_url.parse()?)?;
        if network_details.network.network_name != discovery.network_name {
            anyhow::bail!(
                "Network name mismatch between discovery file and fetched network details"
            )
        }
        nym_network::write_nym_network_details_to_file(&network_details.network)?;
    }
    let network_details = nym_network::setup_nym_network_details(&discovery.network_name)?;
    crate::set_global_network_details(network_details)?;

    // Using discovery, setup nym vpn network details
    nym_vpn_network::setup_nym_vpn_network_details(discovery.nym_vpn_api_url.parse()?)?;

    Ok(())
}
