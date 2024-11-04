// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;
use nym_vpn_network_config::Network;

use crate::{cli::CliArgs, config::GlobalConfigFile, GLOBAL_NETWORK_DETAILS};

fn set_global_network_details(network_details: Network) -> anyhow::Result<()> {
    GLOBAL_NETWORK_DETAILS
        .set(network_details)
        .map_err(|_| anyhow::anyhow!("Failed to set network details"))
}

pub(crate) fn setup_environment(
    global_config_file: &GlobalConfigFile,
    args: &CliArgs,
) -> anyhow::Result<Network> {
    let network_env = if let Some(ref env) = args.config_env_file {
        nym_vpn_lib::nym_config::defaults::setup_env(Some(env));
        let network_details = NymNetworkDetails::new_from_env();
        nym_vpn_network_config::manual_env(&network_details)?
    } else {
        let network_name = global_config_file.network_name.clone();
        let config_path = crate::service::config_dir();

        tracing::info!("Setting up registered networks");
        let networks = nym_vpn_network_config::discover_networks(&config_path)?;
        tracing::info!("Registered networks: {:?}", networks);

        tracing::info!("Setting up environment by discovering the network: {network_name}");
        nym_vpn_network_config::discover_env(&config_path, &network_name)?
    };


    // TODO: pass network_env explicitly instead of relying on being exported to env
    network_env.export_to_env();
    set_global_network_details(network_env.clone())?;
    Ok(network_env)
}
