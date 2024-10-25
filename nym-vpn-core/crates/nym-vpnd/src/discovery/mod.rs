// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, path::PathBuf, time::Duration};

use anyhow::Context;
use nym_vpn_lib::nym_config::defaults::{var_names, NymNetworkDetails};
use tokio_util::sync::CancellationToken;
use url::Url;

const DISCOVERY_FILE: &str = "discovery.json";
const NETWORKS_SUBDIR: &str = "networks";
const DISCOVERY_WELLKNOWN: &str = "https://nymvpn.com/api/public/v1/.wellknown";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct GlobalConfigFile {
    pub(crate) network_name: String,
}

impl Default for GlobalConfigFile {
    fn default() -> Self {
        Self {
            network_name: NymNetworkDetails::default().network_name,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Discovery {
    network_name: String,
    nym_api_url: String,
    nym_vpn_api_url: String,
}

impl Default for Discovery {
    fn default() -> Self {
        let default_network_details = NymNetworkDetails::default();
        Self {
            network_name: default_network_details.network_name,
            nym_api_url: default_network_details
                .endpoints
                .first()
                .and_then(|e| e.api_url.clone())
                .expect("default network details not setup correctly"),
            nym_vpn_api_url: default_network_details
                .nym_vpn_api_url
                .expect("default network details not setup correctly"),
        }
    }
}

// This is the type we fetch remotely from nym-api
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct NetworkDetails {
    pub network: NymNetworkDetails,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct NymVpnNetworkDetails {
    pub nym_vpn_api_url: String,
}

fn discovery_endpoint(network_name: &str) -> anyhow::Result<Url> {
    format!(
        "{}/{}/{}",
        DISCOVERY_WELLKNOWN, network_name, DISCOVERY_FILE
    )
    .parse()
    .map_err(Into::into)
}

fn fetch_discovery(network_name: &str) -> anyhow::Result<Discovery> {
    let url = discovery_endpoint(network_name)?;
    tracing::info!("Fetching nym network discovery from: {}", url);
    reqwest::blocking::get(url.clone())
        .with_context(|| format!("Failed to fetch discovery from {}", url))?
        .json()
        .with_context(|| "Failed to parse discovery")
}

fn discovery_file_path(network_name: &str) -> PathBuf {
    crate::service::config_dir()
        .join(NETWORKS_SUBDIR)
        .join(format!("{}_{}", network_name, DISCOVERY_FILE))
}

fn check_if_discovery_file_exists(network_name: &str) -> bool {
    discovery_file_path(network_name).exists()
}

fn read_discovery_file(network_name: &str) -> anyhow::Result<Discovery> {
    let discovery_path = discovery_file_path(network_name);
    tracing::info!("Reading discovery file from: {}", discovery_path.display());

    let file_str = std::fs::read_to_string(discovery_path)?;
    let network: Discovery = serde_json::from_str(&file_str)?;
    Ok(network)
}

fn write_discovery_to_file(discovery: &Discovery) -> anyhow::Result<()> {
    let discovery_path = discovery_file_path(&discovery.network_name);
    tracing::info!("Writing discovery file to: {}", discovery_path.display());

    // Create parent directories if they don't exist
    if let Some(parent) = discovery_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create parent directories for {:?}",
                discovery_path
            )
        })?;
    }

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&discovery_path)
        .with_context(|| format!("Failed to open discovery file at {:?}", discovery_path))?;

    serde_json::to_writer_pretty(&file, discovery)
        .with_context(|| format!("Failed to write discovery file at {:?}", discovery_path))?;

    Ok(())
}

fn fetch_nym_network_details(nym_api_url: Url) -> anyhow::Result<NetworkDetails> {
    let url = format!("{}/v1/network/details", nym_api_url);
    tracing::info!("Fetching nym network details from: {}", url);
    reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to fetch network details from {}", url))?
        .json()
        .with_context(|| "Failed to parse network details")
}

fn network_details_path(network_name: &str) -> PathBuf {
    crate::service::config_dir()
        .join(NETWORKS_SUBDIR)
        .join(format!("{}.json", network_name))
}

fn check_if_nym_network_details_file_exists(network_name: &str) -> bool {
    network_details_path(network_name).exists()
}

fn read_nym_network_details_from_file(network_name: &str) -> anyhow::Result<NymNetworkDetails> {
    let network_details_path = network_details_path(network_name);
    tracing::info!(
        "Reading network details from: {}",
        network_details_path.display()
    );
    let file_str = std::fs::read_to_string(network_details_path)?;
    let network: NymNetworkDetails = serde_json::from_str(&file_str)?;
    Ok(network)
}

fn write_nym_network_details_to_file(network: &NymNetworkDetails) -> anyhow::Result<()> {
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

fn setup_nym_network_details(network_name: &str) -> anyhow::Result<NymNetworkDetails> {
    let network = read_nym_network_details_from_file(network_name)?;
    export_nym_network_details_to_env(network.clone());
    Ok(network)
}

fn setup_nym_vpn_network_details(nym_vpn_api_url: Url) -> anyhow::Result<NymVpnNetworkDetails> {
    let vpn_network_details = NymVpnNetworkDetails {
        nym_vpn_api_url: nym_vpn_api_url.to_string(),
    };
    export_nym_vpn_network_details_to_env(vpn_network_details.clone());
    Ok(vpn_network_details)
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

fn export_nym_vpn_network_details_to_env(vpn_network_details: NymVpnNetworkDetails) {
    env::set_var(var_names::NYM_VPN_API, vpn_network_details.nym_vpn_api_url);
}

pub fn read_global_config_file() -> anyhow::Result<GlobalConfigFile> {
    let global_config_file_path =
        crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE);

    crate::service::create_config_file(&global_config_file_path, &GlobalConfigFile::default())?;
    crate::service::read_config_file(&global_config_file_path).map_err(Into::into)
}

pub fn write_global_config_file(
    global_config: GlobalConfigFile,
) -> anyhow::Result<GlobalConfigFile> {
    let global_config_file_path =
        crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE);

    crate::service::write_config_file(&global_config_file_path, global_config).map_err(Into::into)
}

pub(crate) fn discover_env(network_name: &str) -> anyhow::Result<()> {
    // Lookup network discovery to bootstrap
    if !check_if_discovery_file_exists(network_name) {
        let discovery = fetch_discovery(network_name)?;
        if discovery.network_name != network_name {
            anyhow::bail!("Network name mismatch between requested and fetched discovery")
        }
        write_discovery_to_file(&discovery)?;
    }

    // If the file is too old, refresh it.
    // TODO: in the future, we should only refresh the discovery file when the tunnel is up.
    // Probably in a background task.
    if let Some(age) = get_age_of_discovery_file(network_name)? {
        if age > MAX_DISCOVERY_AGE {
            refresh_discovery_file(network_name)?;
        }
    }

    let discovery = read_discovery_file(network_name)?;

    // Using discovery, fetch and setup nym network details
    if !check_if_nym_network_details_file_exists(&discovery.network_name) {
        let network_details = fetch_nym_network_details(discovery.nym_api_url.parse()?)?;
        if network_details.network.network_name != discovery.network_name {
            anyhow::bail!(
                "Network name mismatch between discovery file and fetched network details"
            )
        }
        write_nym_network_details_to_file(&network_details.network)?;
    }
    let network_details = setup_nym_network_details(&discovery.network_name)?;
    crate::set_global_network_details(network_details)?;

    // Using discovery, setup nym vpn network details
    setup_nym_vpn_network_details(discovery.nym_vpn_api_url.parse()?)?;

    Ok(())
}

// Refresh the discovery file periodically
const MAX_DISCOVERY_AGE: Duration = Duration::from_secs(60 * 60 * 24);

fn get_age_of_discovery_file(network_name: &str) -> anyhow::Result<Option<Duration>> {
    let discovery_path = discovery_file_path(network_name);
    if !discovery_path.exists() {
        return Ok(None);
    }
    let metadata = std::fs::metadata(discovery_path)?;
    Ok(Some(metadata.modified()?.elapsed()?))
}

fn refresh_discovery_file(network_name: &str) -> anyhow::Result<()> {
    if let Some(age) = get_age_of_discovery_file(network_name)? {
        if age < MAX_DISCOVERY_AGE {
            return Ok(());
        }
    }

    let discovery = fetch_discovery(network_name)?;
    if discovery.network_name != network_name {
        anyhow::bail!("Network name mismatch between requested and fetched discovery")
    }
    write_discovery_to_file(&discovery)?;

    Ok(())
}

// Ideally we only refresh the discovery file when the tunnel is up
// #[allow(unused)]
async fn start_background_discovery_refresh(
    network_name: String,
    cancel_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // Check once an hour
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
        interval.tick().await; // initial tick

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(err) = refresh_discovery_file(&network_name) {
                        tracing::error!("Failed to refresh discovery file: {:?}", err);
                    }
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }
    })
}
