// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::storage::{error::StatsStorageError, sqlite::SqliteStatsStorageManager};
use rand::Rng;
use sqlx::ConnectOptions;
use sqlx_pool_guard::SqlitePoolGuard;
use std::path::Path;
use tracing::{log::LevelFilter, Instrument};

pub mod error;
mod sqlite;

const STATS_DB_FILE_NAME: &str = "stats.db";

#[derive(Debug, Clone)]
pub(crate) struct StatsStorage {
    storage_manager: SqliteStatsStorageManager,
}

impl StatsStorage {
    pub(crate) async fn init<P: AsRef<Path>>(
        base_data_directory: P,
    ) -> Result<Self, StatsStorageError> {
        let database_path = base_data_directory.as_ref().join(STATS_DB_FILE_NAME);

        tracing::debug!("Initing stats storage: {}", database_path.display());

        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&database_path)
            .create_if_missing(true)
            .log_statements(LevelFilter::Trace);

        tracing::debug!("Connecting to the database");
        let connection_pool = SqlitePoolGuard::new(
            database_path.to_path_buf(),
            sqlx::sqlite::SqlitePoolOptions::new()
                .connect_with(opts)
                .await?,
        );

        if let Err(e) = sqlx::migrate!("./migrations").run(&*connection_pool).await {
            connection_pool.close().await;
            return Err(e.into());
        }

        tracing::debug!("Setting file permissions on the database file");
        set_file_permission_owner_rw(&database_path)
            .map_err(|source| StatsStorageError::FilePermissions {
                path: database_path.to_path_buf(),
                source,
            })
            .inspect_err(|err| {
                tracing::error!("Failed to set file permissions: {err:?}");
            })
            .ok();

        Ok(Self {
            storage_manager: SqliteStatsStorageManager::new(connection_pool),
        })
    }

    pub(crate) async fn close(&self) {
        async {
            tracing::debug!("Closing stats storage");
            self.storage_manager.close().await;
            tracing::debug!("Stats storage closed");
        }
        .instrument(tracing::debug_span!("close_stats_storage"))
        .await;
    }

    pub(crate) async fn maybe_init_and_load_seed(&self) -> Result<String, StatsStorageError> {
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

    pub(crate) async fn reset_seed(&self) -> Result<(), StatsStorageError> {
        self.storage_manager.remove_seed().await?;
        self.maybe_init_and_load_seed().await?;
        Ok(())
    }
    pub(crate) async fn remove_seed(&self) -> Result<(), StatsStorageError> {
        self.storage_manager.remove_seed().await
    }
}

fn set_file_permission_owner_rw<P: AsRef<Path>>(_path: P) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    return set_file_permission_owner_rw_unix(_path);

    // Windows permission is set on the parent folder, nothing to do
    #[cfg(windows)]
    return Ok(());

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
