// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use nym_config::defaults::NymNetworkDetails;

use crate::MAX_FILE_AGE;

use super::{Error, NETWORKS_SUBDIR, Result, discovery::Discovery};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
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
            .join(format!("{network_name}.json"))
    }

    pub(super) fn path_is_stale(config_dir: &Path, network_name: &str) -> Result<bool> {
        let file_age = crate::file_age::get_age_of_file(&Self::path(config_dir, network_name))
            .map_err(Error::GetFileAge)?;
        if let Some(age) = file_age {
            Ok(age > MAX_FILE_AGE)
        } else {
            Ok(true)
        }
    }

    pub(super) fn read_from_file(config_dir: &Path, network_name: &str) -> Result<Self> {
        let path = Self::path(config_dir, network_name);
        tracing::debug!("Reading network details from: {}", path.display());

        let file = File::open(&path).map_err(|source| Error::OpenFile {
            path: path.clone(),
            source,
        })?;
        let reader = BufReader::new(file);
        let network: NymNetworkDetails = serde_json::from_reader(reader)
            .map_err(|source| Error::Deserialize { path: path, source })?;
        Ok(Self { network })
    }

    pub(super) fn write_to_file(&self, config_dir: &Path) -> Result<()> {
        let network = &self.network;
        let path = Self::path(config_dir, &network.network_name);

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| Error::CreateParentDirs {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|source| Error::OpenFile {
                path: path.clone(),
                source,
            })?;

        serde_json::to_writer_pretty(&file, network).map_err(|source| Error::WriteFile {
            path: path.clone(),
            source,
        })?;

        Ok(())
    }

    pub(super) async fn ensure_exists(config_dir: &Path, discovery: &Discovery) -> Result<Self> {
        match Self::read_from_file(config_dir, &discovery.network_name) {
            Ok(nym_network) => Ok(nym_network),
            Err(e) if e.should_refresh_file() => {
                if !e.is_file_not_found() {
                    tracing::error!("Failed to read nym network file: {e}");
                }

                let nym_network = discovery.fetch_nym_network_details().await.or_else(|e| {
                    if discovery.network_name == "mainnet" {
                        tracing::warn!(
                            "Failed to fetch remote nym network file: {e}, creating a default one"
                        );
                        Ok(Default::default())
                    } else {
                        tracing::error!(
                            "Failed to fetch remote nym network file: {e}, no default one for {} environment", discovery.network_name
                        );
                        Err(e)
                    }
                })?;

                nym_network.write_to_file(config_dir).inspect_err(|err| {
                    tracing::error!("Failed to write nym network file: {err}");
                })?;

                Ok(nym_network)
            }
            Err(e) => {
                tracing::error!("Failed to read nym network file: {e}");
                Err(e)
            }
        }
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
