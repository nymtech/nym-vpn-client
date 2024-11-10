// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::Context;
use nym_config::defaults::{var_names, NymNetworkDetails};

use super::{discovery::Discovery, MAX_FILE_AGE, NETWORKS_SUBDIR};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NymNetwork {
    pub network: NymNetworkDetails,
}

impl NymNetwork {
    fn path(config_dir: &Path, network_name: &str) -> PathBuf {
        config_dir
            .join(NETWORKS_SUBDIR)
            .join(format!("{}.json", network_name))
    }

    fn path_is_stale(config_dir: &Path, network_name: &str) -> anyhow::Result<bool> {
        if let Some(age) = crate::util::get_age_of_file(&Self::path(config_dir, network_name))? {
            Ok(age > MAX_FILE_AGE)
        } else {
            Ok(true)
        }
    }

    pub(super) fn read_from_file(config_dir: &Path, network_name: &str) -> anyhow::Result<Self> {
        let path = Self::path(config_dir, network_name);
        tracing::debug!("Reading network details from: {}", path.display());
        let file_str = std::fs::read_to_string(path)?;
        let network: NymNetworkDetails = serde_json::from_str(&file_str)?;
        Ok(Self { network })
    }

    pub(super) fn write_to_file(&self, config_dir: &Path) -> anyhow::Result<()> {
        let network = &self.network;
        let path = Self::path(config_dir, &network.network_name);

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directories for {:?}", path))?;
        }

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .with_context(|| format!("Failed to open network details file at {:?}", path))?;

        serde_json::to_writer_pretty(&file, network)
            .with_context(|| format!("Failed to write network details file at {:?}", path))?;

        Ok(())
    }

    pub(super) fn ensure_exists(config_dir: &Path, discovery: &Discovery) -> anyhow::Result<Self> {
        if Self::path_is_stale(config_dir, &discovery.network_name)? {
            discovery
                .fetch_nym_network_details()?
                .write_to_file(config_dir)?;
        }
        Self::read_from_file(config_dir, &discovery.network_name)
    }

    pub(super) fn export_to_env(&self) {
        export_nym_network_details_to_env(self.network.clone())
    }
}

impl From<NymNetworkDetails> for NymNetwork {
    fn from(network: NymNetworkDetails) -> Self {
        Self { network }
    }
}

// TODO: move this to the NymNetworkDetails struct in the nym repo
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nym_network_path() {
        let config_dir = Path::new("/tmp");
        let network_name = "mainnet";
        let path = NymNetwork::path(config_dir, network_name);
        assert_eq!(path, Path::new("/tmp/networks/mainnet.json"));
    }
}
