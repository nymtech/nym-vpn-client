// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use nym_common::trace_err_chain;
use nym_sdk::NymNetworkDetails;

use crate::{Error, PersistentRecord, Result};

type CachedNetworkDetails = PersistentRecord<NymNetworkDetails>;

/// Persistent network details store that keeps track of last modification date
#[derive(Debug)]
pub struct PersistentNetworkDetails {
    cache_dir: PathBuf,
    cached_network_details: CachedNetworkDetails,
}

impl PersistentNetworkDetails {
    /// Create in-memory copy of `PersistentNetworkDetails` that hasn't been persisted yet.
    /// The new copy is marked as stale by default.
    fn new(cache_dir: PathBuf, network_details: NymNetworkDetails) -> Self {
        Self {
            cache_dir,
            cached_network_details: CachedNetworkDetails::stale(network_details),
        }
    }

    /// Create `PersistentNetworkDetails` persisting newly fetched network details and marking them as fresh.
    pub async fn new_with_newly_fetched(
        cache_dir: PathBuf,
        network_details: NymNetworkDetails,
    ) -> Result<Self> {
        let persistent_network_details = Self {
            cache_dir,
            cached_network_details: CachedNetworkDetails::up_to_date(network_details),
        };

        persistent_network_details.write().await?;

        Ok(persistent_network_details)
    }

    /// Create persistent network details from disk cache.
    /// If disk cache is not available, pre-bundled default discovery is used instead and persisted on disk right away but only for mainnet.
    /// Returns `Error::NoDefaultNetworkDetails` when disk cache is empty and if there are no pre-bundled defaults that can be used to set up the store.
    pub async fn new_from_cache(cache_dir: PathBuf, network_name: &str) -> Result<Self> {
        let path = Self::path(&cache_dir, network_name);
        match crate::serialization::deserialize_from_json_file::<_, CachedNetworkDetails>(&path) {
            Ok(cached_network_details) => {
                if cached_network_details.value.network_name == network_name {
                    Ok(Self {
                        cache_dir,
                        cached_network_details,
                    })
                } else {
                    Err(Error::NetworkNameMismatch {
                        expected: network_name.to_owned(),
                        actual: cached_network_details.value.network_name,
                    })
                }
            }
            Err(err) if err.should_overwrite_file() => {
                if !err.is_file_not_found() {
                    trace_err_chain!(err, "failed to deserialize cache");
                }
                if network_name == "mainnet" {
                    let default_network_details = NymNetworkDetails::new_mainnet();

                    if default_network_details.network_name == network_name {
                        let default_persistent_network_details =
                            Self::new(cache_dir, default_network_details);
                        default_persistent_network_details.write().await?;
                        Ok(default_persistent_network_details)
                    } else {
                        Err(Error::NetworkNameMismatch {
                            expected: network_name.to_owned(),
                            actual: default_network_details.network_name,
                        })
                    }
                } else {
                    Err(Error::NoDefaultNetworkDetails(network_name.to_owned()))
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Returns network name referenced by discovery
    pub fn network_name(&self) -> &str {
        &self.cached_network_details.value.network_name
    }

    /// Update network details and persist changes on disk.
    /// The modification timestamp is automatically updated to the current date.
    pub async fn update(&mut self, updated_network_details: NymNetworkDetails) -> Result<()> {
        let stored_network_name = self.network_name();
        if updated_network_details.network_name == stored_network_name {
            self.cached_network_details = CachedNetworkDetails::up_to_date(updated_network_details);
            self.write().await?;
            Ok(())
        } else {
            Err(Error::NetworkNameMismatch {
                expected: stored_network_name.to_owned(),
                actual: updated_network_details.network_name,
            })
        }
    }

    /// Returns current cached network details held in memory.
    pub fn value(&self) -> &NymNetworkDetails {
        &self.cached_network_details.value
    }

    /// Returns true if the value held is considered to be stale and needs to be refreshed
    pub fn is_stale(&self) -> bool {
        self.cached_network_details.is_stale()
    }

    async fn write(&self) -> Result<()> {
        let path = Self::path(&self.cache_dir, self.network_name());
        let parent_dir = path.parent().expect("cannot be without parent!");

        tokio::fs::create_dir_all(parent_dir)
            .await
            .map_err(|source| Error::CreateParentDirs {
                path: parent_dir.to_path_buf(),
                source,
            })?;

        let _file =
            crate::serialization::serialize_to_json_file(path, &self.cached_network_details)?;

        Ok(())
    }

    fn path(cache_dir: &Path, network_name: &str) -> PathBuf {
        cache_dir
            .join(crate::NETWORKS_SUBDIR)
            .join(network_name)
            .join(format!("{network_name}.json"))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_create_from_empty_cache() {
        let cache_dir = tempdir().unwrap();
        let mut persistent_network_details =
            PersistentNetworkDetails::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();
        assert!(persistent_network_details.is_stale());
        assert_eq!(persistent_network_details.value().network_name, "mainnet");

        let new_network_details = persistent_network_details.value().clone();
        persistent_network_details
            .update(new_network_details.clone())
            .await
            .unwrap();
        assert!(!persistent_network_details.is_stale());

        let persistent_network_details =
            PersistentNetworkDetails::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();
        assert_eq!(persistent_network_details.value(), &new_network_details);
    }

    #[tokio::test]
    async fn test_should_fail_creating_empty_cache_for_non_mainnet() {
        let cache_dir = tempdir().unwrap();
        assert!(
            PersistentNetworkDetails::new_from_cache(cache_dir.path().to_path_buf(), "sandbox")
                .await
                .is_err_and(|err| err.is_no_default_network_details())
        );
    }

    #[tokio::test]
    async fn ensure_store_is_written_on_creation() {
        let cache_dir = tempdir().unwrap();
        let _persistent_network_details =
            PersistentNetworkDetails::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();
        let mainnet_store = PersistentNetworkDetails::path(cache_dir.path(), "mainnet");
        assert!(tokio::fs::try_exists(mainnet_store).await.unwrap());
    }

    #[tokio::test]
    async fn test_should_error_init_with_inconsistent_cache() {
        let cache_dir = tempdir().unwrap();
        let persistent_network_details =
            PersistentNetworkDetails::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();
        assert!(persistent_network_details.is_stale());
        assert_eq!(persistent_network_details.value().network_name, "mainnet");

        let mainnet_store = PersistentNetworkDetails::path(cache_dir.path(), "mainnet");
        let sandbox_store = PersistentNetworkDetails::path(cache_dir.path(), "sandbox");

        tokio::fs::create_dir_all(sandbox_store.parent().unwrap())
            .await
            .unwrap();

        tokio::fs::rename(mainnet_store, sandbox_store)
            .await
            .unwrap();

        assert!(
            PersistentNetworkDetails::new_from_cache(cache_dir.path().to_path_buf(), "sandbox")
                .await
                .is_err_and(|err| err.is_inconsistent_network())
        );
    }

    #[tokio::test]
    async fn test_should_prohibit_inconsistent_update() {
        let cache_dir = tempdir().unwrap();
        let mut persistent_network_details =
            PersistentNetworkDetails::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();

        let mut new_network_details = persistent_network_details.value().clone();
        new_network_details.network_name = "sandbox".to_owned();

        assert!(
            persistent_network_details
                .update(new_network_details)
                .await
                .is_err_and(|err| err.is_inconsistent_network())
        );
        assert_eq!(persistent_network_details.network_name(), "mainnet");
    }
}
