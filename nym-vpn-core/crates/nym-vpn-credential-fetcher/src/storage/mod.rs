// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_sqlx_pool_guard::SqlitePoolGuard;

use crate::storage::{
    error::PendingCredentialRequestsStorageError,
    models::{PendingCredentialRequest, PendingCredentialRequestStored},
};

use sqlite::SqliteZkNymRequestsStorageManager;
use sqlx::ConnectOptions;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use time::OffsetDateTime;
use tracing::log::LevelFilter;

pub(crate) mod error;
pub(crate) mod models;
mod sqlite;

// Consider requests older than 60 days as stale
const DEFAULT_STALE_REQUESTS_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 60);

#[derive(Clone)]
pub(crate) struct PendingCredentialRequestsStorage {
    database_path: PathBuf,

    storage_manager: SqliteZkNymRequestsStorageManager,
}

impl PendingCredentialRequestsStorage {
    pub(crate) async fn init<P: AsRef<Path>>(
        data_dir: P,
    ) -> Result<Self, PendingCredentialRequestsStorageError> {
        let database_path = data_dir.as_ref().join("pending_credential_requests.db");

        tracing::debug!(
            "Setting up pending credential requests storage: {}",
            database_path.display()
        );

        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&database_path)
            .create_if_missing(true)
            .log_statements(LevelFilter::Trace);

        tracing::debug!("Connecting to the database");
        let connection_pool = SqlitePoolGuard::new(
            sqlx::sqlite::SqlitePoolOptions::new()
                .connect_with(opts)
                .await?,
        );

        set_file_permission_owner_rw(&database_path)
            .map_err(
                |source| PendingCredentialRequestsStorageError::FilePermissions {
                    path: database_path.to_path_buf(),
                    source,
                },
            )
            .inspect_err(|err| {
                tracing::error!("Failed to set file permissions: {err:?}");
            })
            .ok();

        tracing::debug!("Running migrations");
        if let Err(e) = sqlx::migrate!("./migrations").run(&*connection_pool).await {
            connection_pool.close().await;
            return Err(e.into());
        }

        Ok(Self {
            database_path,
            storage_manager: SqliteZkNymRequestsStorageManager::new(connection_pool),
        })
    }

    pub(crate) async fn close(&self) {
        self.storage_manager.close().await
    }

    pub(crate) async fn reset(self) -> Result<(), PendingCredentialRequestsStorageError> {
        // First we close the storage to ensure that all files are closed
        tracing::debug!("Closing pending credential requests storage");
        self.storage_manager.close().await;

        // Calling close on the storage should be enough to ensure that all files
        // are closed but just to be sure we wait a bit
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Then we remove the database file
        tracing::debug!("Removing pending credential requests storage file");
        tokio::fs::remove_file(&self.database_path)
            .await
            .map_err(PendingCredentialRequestsStorageError::RemoveStorage)?;
        tracing::info!("Removed file: {}", self.database_path.display());

        Ok(())
    }

    pub(crate) async fn get_pending_requests(
        &self,
    ) -> Result<Vec<PendingCredentialRequest>, PendingCredentialRequestsStorageError> {
        self.storage_manager
            .get_pending_requests()
            .await
            .map(|requests| {
                requests
                    .into_iter()
                    .filter_map(|stored| {
                        stored
                            .try_into()
                            .inspect_err(|err| {
                                tracing::error!("Failed to deserialize stored request: {err:?}");
                            })
                            .ok()
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(Into::into)
    }

    pub(crate) async fn insert_pending_request(
        &self,
        pending_request: PendingCredentialRequest,
    ) -> Result<(), PendingCredentialRequestsStorageError> {
        tracing::debug!("Inserting pending request with id: {}", pending_request.id);
        let pending_request = PendingCredentialRequestStored::try_from(pending_request)?;
        self.storage_manager
            .insert_pending_request(
                &pending_request.id,
                pending_request.expiration_date,
                &pending_request.request_info,
            )
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn remove_pending_request(
        &self,
        id: &str,
    ) -> Result<(), PendingCredentialRequestsStorageError> {
        tracing::debug!("Removing pending request with id: {}", id);
        self.storage_manager
            .remove_pending_request(id)
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn clean_up_stale_requests(
        &self,
    ) -> Result<(), PendingCredentialRequestsStorageError> {
        let cutoff = OffsetDateTime::now_utc() - DEFAULT_STALE_REQUESTS_MAX_AGE;
        self.storage_manager
            .remove_stale(cutoff)
            .await
            .map_err(Into::into)
    }
}

#[cfg(windows)]
fn set_file_permission_owner_rw<P: AsRef<Path>>(_path: P) -> Result<(), std::io::Error> {
    // We set permissions for the parent folder instead of the file itself
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn set_file_permission_owner_rw<P: AsRef<Path>>(_path: P) -> Result<(), std::io::Error> {
    tracing::warn!(
        "Setting file permissions on the database file is not yet implemented for this platform!"
    );
    Ok(())
}

#[cfg(unix)]
fn set_file_permission_owner_rw<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;
    tracing::debug!("Setting file permissions on the database file");
    let metadata = std::fs::metadata(&path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(&path, permissions)
}
