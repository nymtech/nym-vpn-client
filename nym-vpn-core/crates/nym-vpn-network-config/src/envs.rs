// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
};

use anyhow::Context;
use itertools::Itertools;

use super::{MAX_FILE_AGE, NETWORKS_SUBDIR};

// TODO: integrate with nym-vpn-api-client

const ENVS_FILE: &str = "envs.json";
const ENVS_WELLKNOWN: &str = "https://nymvpn.com/api/public/v1/.wellknown/envs.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredNetworks {
    inner: HashSet<String>,
}

// Include the generated Default implementation
include!(concat!(env!("OUT_DIR"), "/default_envs.rs"));

impl<'de> serde::de::Deserialize<'de> for RegisteredNetworks {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let registered: HashSet<String> = serde::de::Deserialize::deserialize(deserializer)?;
        Ok(RegisteredNetworks { inner: registered })
    }
}

impl serde::ser::Serialize for RegisteredNetworks {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl fmt::Display for RegisteredNetworks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.iter().format(", "))
    }
}

impl RegisteredNetworks {
    fn path(config_dir: &Path) -> PathBuf {
        config_dir.join(NETWORKS_SUBDIR).join(ENVS_FILE)
    }

    fn path_is_stale(config_dir: &Path) -> anyhow::Result<bool> {
        if let Some(age) = crate::util::get_age_of_file(&Self::path(config_dir))? {
            Ok(age > MAX_FILE_AGE)
        } else {
            Ok(true)
        }
    }

    fn endpoint() -> anyhow::Result<String> {
        ENVS_WELLKNOWN.parse().map_err(Into::into)
    }

    fn fetch() -> anyhow::Result<Self> {
        let url = Self::endpoint()?;
        tracing::info!("Fetching registered networks from: {}", url);

        let response = reqwest::blocking::get(url.clone())
            .inspect_err(|err| tracing::warn!("{}", err))
            .with_context(|| format!("Failed to fetch envs from {}", url))?
            .error_for_status()
            .inspect_err(|err| tracing::warn!("{}", err))
            .with_context(|| "Envs endpoint returned error response".to_owned())?;

        let text_response = response
            .text()
            .inspect_err(|err| tracing::warn!("{}", err))
            .with_context(|| "Failed to read response text")?;
        tracing::debug!("Envs response: {:#?}", text_response);

        serde_json::from_str(&text_response).with_context(|| "Failed to parse envs response")
    }

    fn read_from_file(config_dir: &Path) -> anyhow::Result<Self> {
        let path = Self::path(config_dir);
        tracing::info!(
            "Reading registered networks from file: {:?}",
            path.display()
        );

        let file_str = std::fs::read_to_string(&path)?;
        let registered_networks: RegisteredNetworks = serde_json::from_str(&file_str)?;
        Ok(registered_networks)
    }

    fn write_to_file(&self, config_dir: &Path) -> anyhow::Result<()> {
        let path = Self::path(config_dir);
        tracing::info!("Writing registered networks to file: {:?}", path.display());

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directories for {:?}", path))?;
        }

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .with_context(|| format!("Failed to open envs file: {:?}", path))?;

        serde_json::to_writer_pretty(&file, &self)
            .with_context(|| format!("Failed to write envs file: {:?}", path))?;

        Ok(())
    }

    fn try_update_file(config_dir: &Path) -> anyhow::Result<()> {
        if Self::path_is_stale(config_dir)? {
            Self::fetch()?.write_to_file(config_dir)?;
        }

        Ok(())
    }

    pub(super) fn ensure_exists(config_dir: &Path) -> anyhow::Result<Self> {
        if !Self::path(config_dir).exists() {
            Self::default()
                .write_to_file(config_dir)
                .inspect_err(|err| tracing::warn!("Failed to write default envs file: {err}"))
                .ok();
        }

        // Download the file if it doesn't exists, or if the file is too old, refresh it.
        // TODO: in the future, we should only refresh the discovery file when the tunnel is up.
        // Probably in a background task.

        Self::try_update_file(config_dir)
            .inspect_err(|err| {
                tracing::warn!("Failed to update envs file: {err}");
                tracing::warn!("Attempting to read envs file instead");
            })
            .ok();

        Self::read_from_file(config_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registered_networks_endpoint() {
        let endpoint = RegisteredNetworks::endpoint().unwrap();
        assert_eq!(endpoint, ENVS_WELLKNOWN);
    }

    #[test]
    fn test_registered_networks_serialization() {
        let registered_networks = RegisteredNetworks {
            inner: vec!["mainnet".to_string(), "testnet".to_string()]
                .into_iter()
                .collect(),
        };

        let serialized = serde_json::to_string(&registered_networks).unwrap();
        let deserialized: RegisteredNetworks = serde_json::from_str(&serialized).unwrap();

        assert_eq!(registered_networks, deserialized);
    }

    #[test]
    fn test_registered_networks_default() {
        let registered_networks = RegisteredNetworks::default();
        assert!(registered_networks.inner.contains("mainnet"));
    }

    #[test]
    fn test_registered_networks_fetch() {
        let registered_networks = RegisteredNetworks::fetch().unwrap();
        assert!(registered_networks.inner.contains("mainnet"));
    }

    #[test]
    fn test_registered_networks_write_to_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_dir = temp_dir.path();

        let registered_networks = RegisteredNetworks::default();
        registered_networks.write_to_file(config_dir).unwrap();

        let read_registered_networks = RegisteredNetworks::read_from_file(config_dir).unwrap();
        assert_eq!(registered_networks, read_registered_networks);
    }

    #[test]
    fn test_envs_default_same_as_fetched() {
        let default_envs = RegisteredNetworks::default();
        let fetched_envs = RegisteredNetworks::fetch().unwrap();
        assert_eq!(default_envs, fetched_envs);
    }
}
