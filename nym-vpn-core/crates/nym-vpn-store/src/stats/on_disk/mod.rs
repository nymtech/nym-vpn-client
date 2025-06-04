// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

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

    #[error(" migrate error: {0}")]
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

impl StatsStorage for OnDiskStatsStorage {
    type StorageError = OnDiskStatsStorageError;
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
