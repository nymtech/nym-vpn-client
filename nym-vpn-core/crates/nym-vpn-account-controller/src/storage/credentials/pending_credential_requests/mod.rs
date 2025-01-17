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
use tracing::log::LevelFilter;

use error::PendingCredentialRequestsStorageError;
use models::{PendingCredentialRequest, PendingCredentialRequestStored};

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
            .log_statements(LevelFilter::Info);

        let connection_pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(opts)
            .await?;

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

        sqlx::migrate!("./migrations").run(&connection_pool).await?;

        Ok(Self {
            storage_manager: SqliteZkNymRequestsStorageManager::new(connection_pool),
            database_path: database_path.as_ref().to_path_buf(),
        })
    }

    pub(crate) async fn reset(&mut self) -> Result<(), PendingCredentialRequestsStorageError> {
        // First we close the storage to ensure that all files are closed
        self.storage_manager.close().await;

        // Calling close on the storage should be enough to ensure that all files
        // are closed but just to be sure we wait a bit
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Then we remove the database file
        std::fs::remove_file(&self.database_path)
            .inspect_err(|err| {
                tracing::error!("Failed to remove file: {err:?}");
            })
            .ok();

        // Finally we recreate the storage
        let new_storage_manager = Self::init(&self.database_path).await?;

        self.storage_manager = new_storage_manager.storage_manager;
        self.database_path = new_storage_manager.database_path.clone();

        Ok(())
    }

    pub(crate) async fn insert_pending_request(
        &self,
        pending_request: PendingCredentialRequest,
    ) -> Result<(), PendingCredentialRequestsStorageError> {
        let pending_request = PendingCredentialRequestStored::try_from(pending_request).unwrap();
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
                    .map(|stored| stored.try_into().unwrap())
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
            .map(|request| request.map(|stored| stored.try_into().unwrap()))
            .map_err(Into::into)
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
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(&path, permissions)
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use winapi::um::winnt::FILE_ATTRIBUTE_NORMAL;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .attributes(FILE_ATTRIBUTE_NORMAL)
            .open(&path)?;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
    }
}
