// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use nym_common::trace_err_chain;
use nym_network_defaults::NymNetworkDetails;

use crate::MAX_FILE_AGE;

use super::{Error, Result, discovery::Discovery};

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NymNetwork {
    pub network: NymNetworkDetails,
}

impl NymNetwork {
    pub fn new(network: NymNetworkDetails) -> Self {
        Self { network }
    }

    fn path(config_dir: &Path, network_name: &str) -> PathBuf {
        config_dir
            .join(network_name)
            .join(format!("{network_name}.json"))
    }

    pub(super) fn path_is_stale(config_dir: &Path, network_name: &str) -> Result<bool> {
        let path = Self::path(config_dir, network_name);

        crate::filetime::is_stale_file(&path, MAX_FILE_AGE)
            .map_err(|source| Error::GetFileStaleness { path, source })
    }

    pub(super) fn read_from_file(config_dir: &Path, network_name: &str) -> Result<Self> {
        let path = Self::path(config_dir, network_name);
        tracing::debug!("Reading network details from: {}", path.display());

        let network: NymNetworkDetails = crate::serialization::deserialize_from_json_file(path)?;

        Ok(Self { network })
    }

    pub(super) fn write_to_file(
        &self,
        config_dir: &Path,
        modified_at: Option<SystemTime>,
    ) -> Result<()> {
        let network = &self.network;
        let path = Self::path(config_dir, &network.network_name);

        let file = crate::serialization::serialize_to_json_file(path, network)?;
        if let Some(modified_at) = modified_at
            && let Err(e) = file.set_modified(modified_at)
        {
            tracing::error!("Failed to set modified time for nym network file: {e}");
        }

        Ok(())
    }

    pub(super) async fn ensure_exists(config_dir: &Path, discovery: &Discovery) -> Result<Self> {
        match Self::read_from_file(config_dir, &discovery.network_name) {
            Ok(nym_network) => Ok(nym_network),
            Err(e) if e.should_overwrite_file() => {
                if !e.is_file_not_found() {
                    trace_err_chain!(e, "Failed to read nym network file");
                }

                match discovery.fetch_nym_network_details().await {
                    Ok(nym_network) => {
                        nym_network
                            .write_to_file(config_dir, None)
                            .inspect_err(|err| {
                                trace_err_chain!(err, "Failed to write nym network file");
                            })?;
                        Ok(nym_network)
                    }
                    Err(e) => {
                        if discovery.network_name == "mainnet" {
                            tracing::warn!(
                                "Failed to fetch remote nym network file: {e}, creating a default one"
                            );
                            let default_nym_network = Self::default();
                            let modified_at = SystemTime::now().checked_sub(MAX_FILE_AGE);
                            default_nym_network
                                .write_to_file(config_dir, modified_at)
                                .inspect_err(|err| {
                                    trace_err_chain!(err, "Failed to write nym network file");
                                })?;
                            Ok(default_nym_network)
                        } else {
                            trace_err_chain!(
                                e,
                                "Failed to fetch remote nym network file, no default one for {} environment",
                                discovery.network_name
                            );
                            Err(e)
                        }
                    }
                }
            }
            Err(e) => {
                trace_err_chain!(e, "Failed to read nym network file");
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
