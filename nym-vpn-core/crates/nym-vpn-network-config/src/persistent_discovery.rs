// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use nym_common::trace_err_chain;

use crate::{Error, PersistentRecord, Result, discovery::Discovery};

type CachedDiscovery = PersistentRecord<Box<Discovery>>;

/// Persistent discovery store that keeps track of last modification date
#[derive(Debug)]
pub struct PersistentDiscovery {
    cache_dir: PathBuf,
    cached_discovery: CachedDiscovery,
}

impl PersistentDiscovery {
    /// Create in-memory copy of `PersistentDiscovery` that hasn't been persisted yet.
    /// The new copy is marked as stale by default and is suitable for creating default discoveries bundled with the app.
    fn new(cache_dir: PathBuf, discovery: Discovery) -> Self {
        Self {
            cache_dir,
            cached_discovery: CachedDiscovery::stale(Box::new(discovery)),
        }
    }

    /// Create persistent discovery from disk cache.
    /// If disk cache is not available, pre-bundled default discovery is used instead and persisted on disk right away.
    pub async fn new_from_cache(cache_dir: PathBuf, network_name: &str) -> Result<Self> {
        let path = Self::path(&cache_dir, network_name);
        match crate::serialization::deserialize_from_json_file::<_, CachedDiscovery>(&path) {
            Ok(cached_discovery) => {
                if cached_discovery.value.network_name == network_name {
                    Ok(Self {
                        cache_dir,
                        cached_discovery,
                    })
                } else {
                    Err(Error::NetworkNameMismatch {
                        expected: network_name.to_owned(),
                        actual: cached_discovery.value.network_name,
                    })
                }
            }
            Err(err) if err.should_overwrite_file() => {
                if !err.is_file_not_found() {
                    trace_err_chain!(err, "failed to deserialize cache");
                }

                let default_discovery = Discovery::default_discovery(network_name)
                    .ok_or_else(|| Error::UnknownDiscovery(network_name.to_owned()))?;

                if default_discovery.network_name == network_name {
                    let default_persistent_discovery = Self::new(cache_dir, default_discovery);
                    default_persistent_discovery.write().await?;
                    Ok(default_persistent_discovery)
                } else {
                    Err(Error::NetworkNameMismatch {
                        expected: network_name.to_owned(),
                        actual: default_discovery.network_name,
                    })
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Returns network name referenced by discovery
    pub fn network_name(&self) -> &str {
        &self.cached_discovery.value.network_name
    }

    /// Update discovery and persist changes on disk.
    /// The modification timestamp is automatically updated to the current date.
    pub async fn update(&mut self, updated_discovery: Discovery) -> Result<()> {
        let stored_network_name = self.network_name();
        if updated_discovery.network_name == stored_network_name {
            self.cached_discovery = CachedDiscovery::up_to_date(Box::new(updated_discovery));
            self.write().await?;
            Ok(())
        } else {
            Err(Error::NetworkNameMismatch {
                expected: stored_network_name.to_owned(),
                actual: updated_discovery.network_name,
            })
        }
    }

    /// Returns current discovery held in memory.
    pub fn value(&self) -> &Discovery {
        &self.cached_discovery.value
    }

    /// Returns true if the value held is considered to be stale and needs to be refreshed
    pub fn is_stale(&self) -> bool {
        self.cached_discovery.is_stale()
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

        let _file = crate::serialization::serialize_to_json_file(path, &self.cached_discovery)?;

        Ok(())
    }

    fn path(cache_dir: &Path, network_name: &str) -> PathBuf {
        cache_dir
            .join(crate::NETWORKS_SUBDIR)
            .join(network_name)
            .join(format!("{network_name}_discovery.json"))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_create_from_empty_cache() {
        let cache_dir = tempdir().unwrap();
        let mut persistent_discovery =
            PersistentDiscovery::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();
        assert!(persistent_discovery.is_stale());
        assert_eq!(persistent_discovery.value().network_name, "mainnet");

        let new_discovery = persistent_discovery.value().clone();
        persistent_discovery
            .update(new_discovery.clone())
            .await
            .unwrap();
        assert!(!persistent_discovery.is_stale());

        let persistent_discovery =
            PersistentDiscovery::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();
        assert_eq!(persistent_discovery.value(), &new_discovery);
    }

    #[tokio::test]
    async fn ensure_store_is_written_on_creation() {
        let cache_dir = tempdir().unwrap();
        let _persistent_network_details =
            PersistentDiscovery::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();
        let mainnet_store = PersistentDiscovery::path(cache_dir.path(), "mainnet");
        assert!(tokio::fs::try_exists(mainnet_store).await.unwrap());
    }

    #[tokio::test]
    async fn test_should_error_init_with_inconsistent_cache() {
        let cache_dir = tempdir().unwrap();
        let persistent_discovery =
            PersistentDiscovery::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();
        assert!(persistent_discovery.is_stale());
        assert_eq!(persistent_discovery.value().network_name, "mainnet");

        let mainnet_store = PersistentDiscovery::path(cache_dir.path(), "mainnet");
        let sandbox_store = PersistentDiscovery::path(cache_dir.path(), "sandbox");

        tokio::fs::create_dir_all(sandbox_store.parent().unwrap())
            .await
            .unwrap();

        tokio::fs::rename(mainnet_store, sandbox_store)
            .await
            .unwrap();

        assert!(
            PersistentDiscovery::new_from_cache(cache_dir.path().to_path_buf(), "sandbox")
                .await
                .is_err_and(|err| err.is_inconsistent_network())
        );
    }

    #[tokio::test]
    async fn test_should_prohibit_inconsistent_update() {
        let cache_dir = tempdir().unwrap();
        let mut persistent_discovery =
            PersistentDiscovery::new_from_cache(cache_dir.path().to_path_buf(), "mainnet")
                .await
                .unwrap();

        let mut new_discovery = persistent_discovery.value().clone();
        new_discovery.network_name = "sandbox".to_owned();

        assert!(
            persistent_discovery
                .update(new_discovery)
                .await
                .is_err_and(|err| err.is_inconsistent_network())
        );
        assert_eq!(persistent_discovery.network_name(), "mainnet");
    }
}
