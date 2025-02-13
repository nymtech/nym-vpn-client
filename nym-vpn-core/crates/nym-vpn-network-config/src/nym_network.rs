// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use anyhow::Context;
use nym_config::defaults::NymNetworkDetails;

use crate::util::resolve_nym_network_details;

use super::{discovery::Discovery, MAX_FILE_AGE, NETWORKS_SUBDIR};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NymNetwork {
    pub network: NymNetworkDetails,
}

impl NymNetwork {
    pub fn new(network: NymNetworkDetails) -> Self {
        Self { network }
    }

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
        let mut network: NymNetworkDetails = serde_json::from_str(&file_str)?;
        resolve_nym_network_details(&mut network);
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
        self.network.clone().export_to_env()
    }
}

impl From<NymNetworkDetails> for NymNetwork {
    fn from(network: NymNetworkDetails) -> Self {
        Self { network }
    }
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
