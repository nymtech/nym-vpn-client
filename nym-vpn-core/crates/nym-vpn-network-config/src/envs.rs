// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
};

use crate::{Error, MAX_FILE_AGE, NETWORKS_SUBDIR, Result, discovery::Discovery};
use itertools::Itertools;
use nym_common::trace_err_chain;
use nym_sdk::UserAgent;
use nym_vpn_api_client::{ResolverOverrides, VpnApiClient, api_urls_to_urls};

// TODO: integrate with nym-vpn-api-client

const ENVS_FILE: &str = "envs.json";
static DEFAULT_ENVS_JSON: &[u8] = include_bytes!("../default/envs.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredNetworks {
    inner: HashSet<String>,
}

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

    fn path_is_stale(config_dir: &Path) -> Result<bool> {
        let path = Self::path(config_dir);

        crate::filetime::is_stale_file(&path, MAX_FILE_AGE)
            .map_err(|source| Error::GetFileStaleness { path, source })
    }

    async fn fetch() -> Result<Self> {
        tracing::debug!("Fetching registered networks");

        // Spawn the root task
        let api_urls = Discovery::default_vpn_api_urls();

        let urls = api_urls_to_urls(api_urls).map_err(Error::CreateVpnApiClient)?;

        let resolver_overrides = ResolverOverrides::from_urls(&urls)
            .await
            .map_err(Error::CreateVpnApiClient)?;

        let inner = VpnApiClient::new(
            urls,
            UserAgent {
                application: String::new(),
                version: String::new(),
                platform: String::new(),
                git_commit: String::new(),
            },
            Some(&resolver_overrides),
        )
        .await
        .map_err(Error::CreateVpnApiClient)?
        .get_wellknown_envs()
        .await
        .map_err(Error::GetWellKnownEnvs)?;
        tracing::debug!("Envs response: {:#?}", inner);

        Ok(Self { inner })
    }

    fn read_from_file(config_dir: &Path) -> Result<Self> {
        let path = Self::path(config_dir);
        tracing::debug!(
            "Reading registered networks from file: {:?}",
            path.display()
        );

        crate::serialization::deserialize_from_json_file(path)
    }

    fn write_to_file(&self, config_dir: &Path) -> Result<()> {
        let path = Self::path(config_dir);
        tracing::debug!("Writing registered networks to file: {}", path.display());

        crate::serialization::serialize_to_json_file(&path, self)
    }

    pub(super) async fn try_update_file(config_dir: &Path) -> Result<()> {
        if Self::path_is_stale(config_dir)? {
            Self::fetch().await?.write_to_file(config_dir)?;
        }

        Ok(())
    }

    pub(super) async fn ensure_exists(config_dir: &Path) -> Result<Self> {
        match Self::read_from_file(config_dir) {
            Ok(registered_networks) => Ok(registered_networks),
            Err(e) if e.should_overwrite_file() => {
                if !e.is_file_not_found() {
                    trace_err_chain!(e, "Failed to read registered networks file");
                }

                let default_envs = Self::default();
                default_envs.write_to_file(config_dir).inspect_err(|err| {
                    trace_err_chain!(err, "Failed to write default envs file");
                })?;

                Ok(default_envs)
            }
            Err(e) => {
                trace_err_chain!(e, "Failed to read registered networks file");
                Err(e)
            }
        }
    }
}

impl Default for RegisteredNetworks {
    fn default() -> Self {
        #[allow(clippy::expect_used)]
        serde_json::from_slice(DEFAULT_ENVS_JSON).expect("Failed to parse default envs JSON")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn test_registered_networks_fetch() {
        let registered_networks = RegisteredNetworks::fetch().await.unwrap();
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

    #[tokio::test]
    async fn test_envs_default_same_as_fetched() {
        let default_envs = RegisteredNetworks::default();
        let fetched_envs = RegisteredNetworks::fetch().await.unwrap();
        assert_eq!(default_envs, fetched_envs);
    }
}
