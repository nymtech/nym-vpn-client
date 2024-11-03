// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use anyhow::Context;
use nym_config::defaults::NymNetworkDetails;
use url::Url;

use super::{nym_network::NymNetwork, MAX_FILE_AGE, NETWORKS_SUBDIR};

// TODO: integrate with nym-vpn-api-client

const DISCOVERY_FILE: &str = "discovery.json";
const DISCOVERY_WELLKNOWN: &str = "https://nymvpn.com/api/public/v1/.wellknown";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Discovery {
    pub(super) network_name: String,
    pub(super) nym_api_url: Url,
    pub(super) nym_vpn_api_url: Url,
}

impl Discovery {
    fn path(config_dir: &Path, network_name: &str) -> PathBuf {
        config_dir
            .join(NETWORKS_SUBDIR)
            .join(format!("{}_{}", network_name, DISCOVERY_FILE))
    }

    pub(super) fn path_is_stale(config_dir: &Path, network_name: &str) -> anyhow::Result<bool> {
        if let Some(age) = crate::util::get_age_of_file(&Self::path(config_dir, network_name))? {
            Ok(age > MAX_FILE_AGE)
        } else {
            Ok(true)
        }
    }

    fn endpoint(network_name: &str) -> anyhow::Result<Url> {
        format!(
            "{}/{}/{}",
            DISCOVERY_WELLKNOWN, network_name, DISCOVERY_FILE
        )
        .parse()
        .map_err(Into::into)
    }

    pub fn fetch(network_name: &str) -> anyhow::Result<Self> {
        let discovery: DiscoveryResponse = {
            let url = Self::endpoint(network_name)?;

            tracing::info!("Fetching nym network discovery from: {}", url);
            let response = reqwest::blocking::get(url.clone())
                .with_context(|| format!("Failed to fetch discovery from {}", url))?
                .error_for_status()
                .with_context(|| "Discovery endpoint returned error response".to_owned())?;

            let text_response = response
                .text()
                .with_context(|| "Failed to read response text")?;
            tracing::debug!("Discovery response: {:#?}", text_response);

            serde_json::from_str(&text_response)
                .with_context(|| "Failed to parse discovery response")
        }?;
        if discovery.network_name != network_name {
            anyhow::bail!("Network name mismatch between requested and fetched discovery")
        }
        discovery.try_into()
    }

    pub(super) fn read_from_file(config_dir: &Path, network_name: &str) -> anyhow::Result<Self> {
        let path = Self::path(config_dir, network_name);
        tracing::info!("Reading discovery file from: {}", path.display());

        let file_str = std::fs::read_to_string(path)?;
        let network: Discovery = serde_json::from_str(&file_str)?;
        Ok(network)
    }

    pub(super) fn write_to_file(&self, config_dir: &Path) -> anyhow::Result<()> {
        let path = Self::path(config_dir, &self.network_name);
        tracing::info!("Writing discovery file to: {}", path.display());

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
            .with_context(|| format!("Failed to open discovery file at {:?}", path))?;

        serde_json::to_writer_pretty(&file, self)
            .with_context(|| format!("Failed to write discovery file at {:?}", path))?;

        Ok(())
    }

    pub(super) fn ensure_exists(config_dir: &Path, network_name: &str) -> anyhow::Result<Self> {
        // Download the file if it doesn't exists, or if the file is too old, refresh it.
        // TODO: in the future, we should only refresh the discovery file when the tunnel is up.
        // Probably in a background task.
        if Self::path_is_stale(config_dir, network_name)? {
            Self::fetch(network_name)?.write_to_file(config_dir)?;
        }

        Self::read_from_file(config_dir, network_name)
    }

    pub fn fetch_nym_network_details(&self) -> anyhow::Result<NymNetwork> {
        let url = format!("{}/v1/network/details", self.nym_api_url);
        tracing::info!("Fetching nym network details from: {}", url);
        let network_details: NymNetworkDetailsResponse = reqwest::blocking::get(&url)
            .with_context(|| format!("Failed to fetch network details from {}", url))?
            .json()
            .with_context(|| "Failed to parse network details")?;
        if network_details.network.network_name != self.network_name {
            anyhow::bail!("Network name mismatch between requested and fetched network details")
        }
        Ok(NymNetwork {
            network: network_details.network,
        })
    }
}

impl Default for Discovery {
    fn default() -> Self {
        let default_network_details = NymNetworkDetails::default();
        Self {
            network_name: default_network_details.network_name,
            nym_api_url: default_network_details
                .endpoints
                .first()
                .and_then(|e| e.api_url().clone())
                .expect("default network details not setup correctly"),
            nym_vpn_api_url: default_network_details
                .nym_vpn_api_url
                .map(|url| {
                    url.parse()
                        .expect("default network details not setup correctly")
                })
                .expect("default network details not setup correctly"),
        }
    }
}

// The response type we fetch from the discovery endpoint
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct DiscoveryResponse {
    network_name: String,
    nym_api_url: String,
    nym_vpn_api_url: String,
}

// The response type we fetch from the network details endpoint. This will be added to and exported
// from nym-api-requests.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct NymNetworkDetailsResponse {
    network: NymNetworkDetails,
}

impl TryFrom<DiscoveryResponse> for Discovery {
    type Error = anyhow::Error;

    fn try_from(discovery: DiscoveryResponse) -> anyhow::Result<Self> {
        Ok(Self {
            network_name: discovery.network_name,
            nym_api_url: discovery.nym_api_url.parse()?,
            nym_vpn_api_url: discovery.nym_vpn_api_url.parse()?,
        })
    }
}
