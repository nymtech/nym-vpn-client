// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, path::PathBuf};

use anyhow::Context;
use nym_vpn_lib::nym_config::defaults::{var_names, NymNetworkDetails};
use url::Url;

fn network_details_path(network_name: &str) -> PathBuf {
    crate::service::config_dir()
        .join(super::NETWORKS_SUBDIR)
        .join(format!("{}.json", network_name))
}

// This is the type we fetch remotely from nym-api
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct NetworkDetails {
    pub(super) network: NymNetworkDetails,
}

pub(super) fn fetch_nym_network_details(nym_api_url: Url) -> anyhow::Result<NetworkDetails> {
    let url = format!("{}/v1/network/details", nym_api_url);
    tracing::info!("Fetching nym network details from: {}", url);
    reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to fetch network details from {}", url))?
        .json()
        .with_context(|| "Failed to parse network details")
}

pub(super) fn check_if_nym_network_details_file_exists(network_name: &str) -> bool {
    network_details_path(network_name).exists()
}

fn read_nym_network_details_from_file(
    network_name: &str,
) -> anyhow::Result<NymNetworkDetails> {
    let network_details_path = network_details_path(network_name);
    tracing::info!(
        "Reading network details from: {}",
        network_details_path.display()
    );
    let file_str = std::fs::read_to_string(network_details_path)?;
    let network: NymNetworkDetails = serde_json::from_str(&file_str)?;
    Ok(network)
}

pub(super) fn write_nym_network_details_to_file(network: &NymNetworkDetails) -> anyhow::Result<()> {
    let network_details_path = network_details_path(&network.network_name);

    // Create parent directories if they don't exist
    if let Some(parent) = network_details_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create parent directories for {:?}",
                network_details_path
            )
        })?;
    }

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&network_details_path)
        .with_context(|| {
            format!(
                "Failed to open network details file at {:?}",
                network_details_path
            )
        })?;

    serde_json::to_writer_pretty(&file, network).with_context(|| {
        format!(
            "Failed to write network details file at {:?}",
            network_details_path
        )
    })?;

    Ok(())
}

pub(super) fn setup_nym_network_details(network_name: &str) -> anyhow::Result<NymNetworkDetails> {
    let network = read_nym_network_details_from_file(network_name)?;
    export_nym_network_details_to_env(network.clone());
    Ok(network)
}

fn export_nym_network_details_to_env(network_details: NymNetworkDetails) {
    fn set_optional_var(var_name: &str, value: Option<String>) {
        if let Some(value) = value {
            env::set_var(var_name, value);
        }
    }

    env::set_var(var_names::NETWORK_NAME, network_details.network_name);
    env::set_var(
        var_names::BECH32_PREFIX,
        network_details.chain_details.bech32_account_prefix,
    );

    env::set_var(
        var_names::MIX_DENOM,
        network_details.chain_details.mix_denom.base,
    );
    env::set_var(
        var_names::MIX_DENOM_DISPLAY,
        network_details.chain_details.mix_denom.display,
    );

    env::set_var(
        var_names::STAKE_DENOM,
        network_details.chain_details.stake_denom.base,
    );
    env::set_var(
        var_names::STAKE_DENOM_DISPLAY,
        network_details.chain_details.stake_denom.display,
    );

    env::set_var(
        var_names::DENOMS_EXPONENT,
        network_details
            .chain_details
            .mix_denom
            .display_exponent
            .to_string(),
    );

    env::set_var(
        var_names::NYXD,
        network_details.endpoints.first().unwrap().nyxd_url.clone(),
    );
    set_optional_var(
        var_names::NYM_API,
        network_details.endpoints.first().unwrap().api_url.clone(),
    );
    set_optional_var(
        var_names::NYXD_WEBSOCKET,
        network_details
            .endpoints
            .first()
            .unwrap()
            .websocket_url
            .clone(),
    );

    set_optional_var(
        var_names::MIXNET_CONTRACT_ADDRESS,
        network_details.contracts.mixnet_contract_address,
    );
    set_optional_var(
        var_names::VESTING_CONTRACT_ADDRESS,
        network_details.contracts.vesting_contract_address,
    );
    set_optional_var(
        var_names::ECASH_CONTRACT_ADDRESS,
        network_details.contracts.ecash_contract_address,
    );
    set_optional_var(
        var_names::GROUP_CONTRACT_ADDRESS,
        network_details.contracts.group_contract_address,
    );
    set_optional_var(
        var_names::MULTISIG_CONTRACT_ADDRESS,
        network_details.contracts.multisig_contract_address,
    );
    set_optional_var(
        var_names::COCONUT_DKG_CONTRACT_ADDRESS,
        network_details.contracts.coconut_dkg_contract_address,
    );

    set_optional_var(var_names::EXPLORER_API, network_details.explorer_api);
    set_optional_var(var_names::NYM_VPN_API, network_details.nym_vpn_api_url);
}
