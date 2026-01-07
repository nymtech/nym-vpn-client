// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::config::GlobalConfig;
use nym_vpn_network_config::Network;

pub async fn setup_environment(global_config_file: &GlobalConfig) -> anyhow::Result<Network> {
    let network_name = global_config_file.network_name.clone();
    let config_path = crate::paths::config_dir();

    tracing::debug!("Setting up registered networks");
    let networks = nym_vpn_network_config::discover_networks(&config_path).await?;
    tracing::debug!("Registered networks: {}", networks);

    tracing::info!("Setting up environment by discovering the network: {network_name}");
    let network_env = nym_vpn_network_config::discover_env(&config_path, &network_name).await?;

    // TODO: we need to export to env here to bridge the gap to older code.
    network_env.export_to_env();
    Ok(network_env)
}
