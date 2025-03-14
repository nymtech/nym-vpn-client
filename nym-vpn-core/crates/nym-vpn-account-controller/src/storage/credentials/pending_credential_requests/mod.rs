// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod error;
pub mod models;

mod sqlite;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use sqlite::SqliteZkNymRequestsStorageManager;
use sqlx::ConnectOptions;
use time::OffsetDateTime;
use tracing::log::LevelFilter;

use error::PendingCredentialRequestsStorageError;
use models::{PendingCredentialRequest, PendingCredentialRequestStored};

// Consider requests older than 60 days as stale
const DEFAULT_STALE_REQUESTS_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 60);

#[derive(Clone)]
pub(crate) struct PendingCredentialRequestsStorage {
    storage_manager: SqliteZkNymRequestsStorageManager,
    database_path: PathBuf,
}

impl PendingCredentialRequestsStorage {
    pub(crate) async fn init<P: AsRef<Path>>(
        database_path: P,
    ) -> Result<Self, PendingCredentialRequestsStorageError> {
        tracing::info!(
            "Setting up pending credential requests storage: {:?}",
            database_path.as_ref().as_os_str()
        );

        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&database_path)
            .create_if_missing(true)
            .log_statements(LevelFilter::Trace);

        tracing::debug!("Connecting to the database");
        let connection_pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(opts)
            .await?;

        tracing::debug!("Setting file permissions on the database file");
        set_file_permission_owner_rw(&database_path)
            .map_err(
                |source| PendingCredentialRequestsStorageError::FilePermissions {
                    path: database_path.as_ref().to_path_buf(),
                    source,
                },
            )
            .inspect_err(|err| {
                tracing::error!("Failed to set file permissions: {err:?}");
            })
            .ok();

        tracing::debug!("Running migrations");
        sqlx::migrate!("./migrations").run(&connection_pool).await?;

        Ok(Self {
            storage_manager: SqliteZkNymRequestsStorageManager::new(connection_pool),
            database_path: database_path.as_ref().to_path_buf(),
        })
    }

    pub(crate) async fn reset(&mut self) -> Result<(), PendingCredentialRequestsStorageError> {
        // First we close the storage to ensure that all files are closed
        tracing::debug!("Closing pending credential requests storage");
        self.storage_manager.close().await;

        // Calling close on the storage should be enough to ensure that all files
        // are closed but just to be sure we wait a bit
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Then we remove the database file
        tracing::debug!("Removing pending credential requests storage file");
        std::fs::remove_file(&self.database_path)
            .map_err(PendingCredentialRequestsStorageError::RemoveStorage)?;
        tracing::info!("Removed file: {}", self.database_path.display());

        // Finally we recreate the storage
        tracing::debug!("Recreating pending credential requests storage");
        let new_storage_manager = Self::init(&self.database_path).await?;

        tracing::debug!("Pending credential requests storage reset completed");
        *self = new_storage_manager;

        Ok(())
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

    pub(crate) async fn insert_pending_request(
        &self,
        pending_request: PendingCredentialRequest,
    ) -> Result<(), PendingCredentialRequestsStorageError> {
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

    pub(crate) async fn get_pending_request_by_id(
        &self,
        id: &str,
    ) -> Result<Option<PendingCredentialRequest>, PendingCredentialRequestsStorageError> {
        self.storage_manager
            .get_pending_request_by_id(id)
            .await
            .map(|request| {
                request
                    .map(|stored| {
                        stored
                            .try_into()
                            .inspect_err(|err| {
                                tracing::error!("Failed to deserialize stored request: {err:?}");
                            })
                            .map_err(Into::into)
                    })
                    .transpose()
            })
            .map_err(PendingCredentialRequestsStorageError::from)?
    }

    pub(crate) async fn remove_pending_request(
        &self,
        id: &str,
    ) -> Result<(), PendingCredentialRequestsStorageError> {
        self.storage_manager
            .remove_pending_request(id)
            .await
            .map_err(Into::into)
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
