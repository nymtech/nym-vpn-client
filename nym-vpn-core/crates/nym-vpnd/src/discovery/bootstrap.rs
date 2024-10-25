// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;
use url::Url;

const DISCOVERY_FILE: &str = "discovery.json";
const DISCOVERY_WELLKNOWN: &str = "https://nymvpn.com/api/public/v1/.wellknown";

// Refresh the discovery file periodically
pub(super) const MAX_DISCOVERY_AGE: Duration = Duration::from_secs(60 * 60 * 24);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Discovery {
    pub network_name: String,
    pub nym_api_url: String,
    pub nym_vpn_api_url: String,
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

fn discovery_file_path(network_name: &str) -> PathBuf {
    crate::service::config_dir()
        .join(super::NETWORKS_SUBDIR)
        .join(format!("{}_{}", network_name, DISCOVERY_FILE))
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

fn check_if_discovery_file_exists(network_name: &str) -> bool {
    discovery_file_path(network_name).exists()
}

pub(super) fn read_discovery_file(network_name: &str) -> anyhow::Result<Discovery> {
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

pub(super) fn download_discovery_to_file(network_name: &str) -> anyhow::Result<()> {
    let discovery = fetch_discovery(network_name)?;
    if discovery.network_name != network_name {
        anyhow::bail!("Network name mismatch between requested and fetched discovery")
    }
    write_discovery_to_file(&discovery)
}

pub(super) fn get_age_of_discovery_file(network_name: &str) -> anyhow::Result<Option<Duration>> {
    let discovery_path = super::bootstrap::discovery_file_path(network_name);
    if !discovery_path.exists() {
        return Ok(None);
    }
    let metadata = std::fs::metadata(discovery_path)?;
    Ok(Some(metadata.modified()?.elapsed()?))
}

pub(super) fn is_time_to_refresh_discovery_file(network_name: &str) -> anyhow::Result<bool> {
    if let Some(age) = get_age_of_discovery_file(network_name)? {
        Ok(age > MAX_DISCOVERY_AGE)
    } else {
        Ok(true)
    }
}

pub(super) fn download_or_refresh_discovery_file(network_name: &str) -> anyhow::Result<()> {
    if !check_if_discovery_file_exists(network_name) {
        download_discovery_to_file(network_name)?;
    }

    // If the file is too old, refresh it.
    // TODO: in the future, we should only refresh the discovery file when the tunnel is up.
    // Probably in a background task.
    if is_time_to_refresh_discovery_file(network_name)? {
        download_discovery_to_file(network_name)?;
    }

    Ok(())
}
