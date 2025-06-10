// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use rand::Rng;
use sqlx::ConnectOptions;
use std::path::{Path, PathBuf};
use tracing::log::LevelFilter;

use super::{StatsStorage, StatsStorageError};
use sqlite::SqliteStatsStorageManager;

mod sqlite;

#[derive(Debug, thiserror::Error)]
pub enum OnDiskStatsStorageError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migrate error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("file permissions error for {path:?}: {source}")]
    FilePermissions {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl StatsStorageError for OnDiskStatsStorageError {}

pub struct OnDiskStatsStorage {
    storage_manager: SqliteStatsStorageManager,
}

impl OnDiskStatsStorage {
    pub fn new(database_path: PathBuf) -> Self {
        tracing::debug!("Initing stats storage: {}", database_path.display());

        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&database_path)
            .create_if_missing(true)
            .log_statements(LevelFilter::Trace);

        tracing::debug!("Connecting to the database");
        let connection_pool = sqlx::sqlite::SqlitePoolOptions::new().connect_lazy_with(opts);

        tracing::debug!("Setting file permissions on the database file");
        set_file_permission_owner_rw(&database_path)
            .map_err(|source| OnDiskStatsStorageError::FilePermissions {
                path: database_path.to_path_buf(),
                source,
            })
            .inspect_err(|err| {
                tracing::error!("Failed to set file permissions: {err:?}");
            })
            .ok();
        Self {
            storage_manager: SqliteStatsStorageManager::new(connection_pool),
        }
    }

    pub async fn init(&mut self) -> Result<(), OnDiskStatsStorageError> {
        Ok(self.storage_manager.migrate().await?)
    }
}

fn set_file_permission_owner_rw<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    return set_file_permission_owner_rw_unix(path);

    #[cfg(windows)]
    return set_file_permission_owner_rw_windows(path);

    #[cfg(not(any(unix, windows)))]
    {
        tracing::warn!("Setting file permissions is not yet implemented for this platform!");
        Ok(())
    }
}

#[cfg(unix)]
fn set_file_permission_owner_rw_unix<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(&path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(&path, permissions)
}

#[cfg(windows)]
fn set_file_permission_owner_rw_windows<P: AsRef<Path>>(_path: P) -> Result<(), std::io::Error> {
    tracing::info!("Setting file permissions on Windows is not yet implemented!");
    Ok(())
}

#[async_trait::async_trait]
impl StatsStorage for OnDiskStatsStorage {
    type StorageError = OnDiskStatsStorageError;

    async fn maybe_init_and_load_stats_seed(&self) -> Result<String, Self::StorageError> {
        match self.storage_manager.load_seed().await {
            Ok(Some(seed)) => Ok(seed),
            Ok(None) => {
                // we don't need anything crypto secure here
                let seed: String = rand::thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(20)
                    .map(char::from)
                    .collect();
                self.storage_manager.set_seed(seed.clone()).await?;
                Ok(seed)
            }
            Err(e) => Err(e),
        }
    }

    async fn reset_stats_seed(&self) -> Result<String, Self::StorageError> {
        self.storage_manager.remove_seed().await?;
        self.maybe_init_and_load_stats_seed().await
    }
    async fn remove_stats_seed(&self) -> Result<(), Self::StorageError> {
        self.storage_manager.remove_seed().await
    }
}
