// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use nym_common::trace_err_chain;

use crate::{Error, PersistentRecord, RegisteredNetworks, Result};

type CachedEnvs = PersistentRecord<RegisteredNetworks>;

/// Persistent network environments store that keeps track of last modification date
#[derive(Debug)]
pub struct PersistentEnvs {
    cache_dir: PathBuf,
    cached_envs: CachedEnvs,
}

impl PersistentEnvs {
    /// Create in-memory copy of `PersistentDiscovery` that hasn't been persisted yet.
    /// The new copy is marked as stale by default and is suitable for creating default discoveries bundled with the app.
    fn new(cache_dir: PathBuf, envs: RegisteredNetworks) -> Self {
        Self {
            cache_dir,
            cached_envs: CachedEnvs::stale(envs),
        }
    }

    /// Create persistent discovery from disk cache.
    /// If disk cache is not available, pre-bundled default discovery is used instead and persisted on disk right away.
    pub async fn new_from_cache(cache_dir: PathBuf) -> Result<Self> {
        let path = Self::path(&cache_dir);
        match crate::serialization::deserialize_from_json_file::<_, CachedEnvs>(&path) {
            Ok(cached_envs) => Ok(Self {
                cache_dir,
                cached_envs,
            }),
            Err(err) if err.should_overwrite_file() => {
                if !err.is_file_not_found() {
                    trace_err_chain!(err, "failed to deserialize cache");
                }
                let default_persistent_envs = Self::new(cache_dir, RegisteredNetworks::default());
                default_persistent_envs.write().await?;
                Ok(default_persistent_envs)
            }
            Err(err) => Err(err),
        }
    }

    /// Update discovery and persist changes on disk.
    /// The modification timestamp is automatically updated to the current date.
    pub async fn update(&mut self, updated_networks: RegisteredNetworks) -> Result<()> {
        self.cached_envs = CachedEnvs::up_to_date(updated_networks);
        self.write().await?;
        Ok(())
    }

    /// Returns current registered networks held in memory.
    pub fn value(&self) -> &RegisteredNetworks {
        &self.cached_envs.value
    }

    /// Returns true if the value held is considered to be stale and needs to be refreshed
    pub fn is_stale(&self) -> bool {
        self.cached_envs.is_stale()
    }

    async fn write(&self) -> Result<()> {
        let path = Self::path(&self.cache_dir);
        let parent_dir = path.parent().expect("cannot be without parent!");

        tokio::fs::create_dir_all(parent_dir)
            .await
            .map_err(|source| Error::CreateParentDirs {
                path: parent_dir.to_path_buf(),
                source,
            })?;

        let _file = crate::serialization::serialize_to_json_file(path, &self.cached_envs)?;

        Ok(())
    }

    fn path(cache_dir: &Path) -> PathBuf {
        cache_dir.join(crate::NETWORKS_SUBDIR).join("envs.json")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_create_from_empty_cache() {
        let cache_dir = tempdir().unwrap();
        let mut persistent_envs = PersistentEnvs::new_from_cache(cache_dir.path().to_path_buf())
            .await
            .unwrap();

        assert!(persistent_envs.is_stale());
        assert!(persistent_envs.value().names().contains("mainnet"));

        let updated_networks =
            RegisteredNetworks::new(HashSet::from_iter(["testnet".to_owned()].into_iter()));
        persistent_envs
            .update(updated_networks.clone())
            .await
            .unwrap();
        assert!(!persistent_envs.is_stale());

        let persistent_envs = PersistentEnvs::new_from_cache(cache_dir.path().to_path_buf())
            .await
            .unwrap();
        assert!(!persistent_envs.is_stale());
        assert_eq!(persistent_envs.value(), &updated_networks);
    }

    #[tokio::test]
    async fn ensure_store_is_written_on_creation() {
        let cache_dir = tempdir().unwrap();
        let _persistent_envs = PersistentEnvs::new_from_cache(cache_dir.path().to_path_buf())
            .await
            .unwrap();
        let cache_file = PersistentEnvs::path(cache_dir.path());
        assert!(tokio::fs::try_exists(cache_file).await.unwrap());
    }
}
