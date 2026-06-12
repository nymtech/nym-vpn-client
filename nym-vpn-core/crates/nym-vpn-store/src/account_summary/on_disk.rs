// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use nym_vpn_lib_types::VpnAccountSummary;

use super::AccountSummaryStorage;

#[derive(Debug, thiserror::Error)]
pub enum OnDiskAccountSummaryStorageError {
    #[error("failed to read account summary file")]
    ReadError(#[source] std::io::Error),

    #[error("failed to write account summary file")]
    WriteError(#[source] std::io::Error),

    #[error("failed to create account summary directory")]
    CreateDirError(#[source] std::io::Error),

    #[error("failed to remove account summary file")]
    RemoveError(#[source] std::io::Error),

    #[error("failed to serialize account summary")]
    SerializeError(#[source] serde_json::Error),

    #[error("failed to deserialize account summary")]
    DeserializeError(#[source] serde_json::Error),
}

pub struct OnDiskAccountSummaryStorage {
    path: PathBuf,
}

impl OnDiskAccountSummaryStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait::async_trait]
impl AccountSummaryStorage for OnDiskAccountSummaryStorage {
    type StorageError = OnDiskAccountSummaryStorageError;

    async fn load_summary(&self) -> Result<Option<VpnAccountSummary>, Self::StorageError> {
        let bytes = match tokio::fs::read(&self.path).await {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(OnDiskAccountSummaryStorageError::ReadError(err)),
        };

        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(OnDiskAccountSummaryStorageError::DeserializeError)
    }

    async fn store_summary(&self, account: VpnAccountSummary) -> Result<(), Self::StorageError> {
        let bytes = serde_json::to_vec(&account)
            .map_err(OnDiskAccountSummaryStorageError::SerializeError)?;

        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(OnDiskAccountSummaryStorageError::CreateDirError)?;
        }

        tokio::fs::write(&self.path, bytes)
            .await
            .map_err(OnDiskAccountSummaryStorageError::WriteError)
    }

    async fn remove_summary(&self) -> Result<(), Self::StorageError> {
        match tokio::fs::remove_file(&self.path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(OnDiskAccountSummaryStorageError::RemoveError(err)),
        }
    }
}
