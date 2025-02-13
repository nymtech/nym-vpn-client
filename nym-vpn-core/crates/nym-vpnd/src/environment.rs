// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;
use nym_vpn_network_config::Network;

use crate::{cli::CliArgs, config::GlobalConfigFile};

pub(crate) fn setup_environment(
    global_config_file: &GlobalConfigFile,
    args: &CliArgs,
) -> anyhow::Result<Network> {
    let network_env = if let Some(ref env) = args.config_env_file {
        nym_vpn_lib::nym_config::defaults::setup_env(Some(env));
        let network_details = NymNetworkDetails::new_from_env();
        // resolve_nym_network_details(&mut network_details);
        nym_vpn_network_config::manual_env(&network_details)?
    } else {
        let network_name = global_config_file.network_name.clone();
        let config_path = crate::service::config_dir();

        tracing::debug!("Setting up registered networks");
        let networks = nym_vpn_network_config::discover_networks(&config_path)?;
        tracing::debug!("Registered networks: {}", networks);

        tracing::info!("Setting up environment by discovering the network: {network_name}");
        nym_vpn_network_config::discover_env(&config_path, &network_name)?
    };

    // TODO: we need to export to env here to bridge the gap to older code.
    network_env.export_to_env();
    Ok(network_env)
}
