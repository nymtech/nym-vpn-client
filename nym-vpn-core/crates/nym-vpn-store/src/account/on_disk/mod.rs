// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod legacy;
mod v1;

#[cfg(test)]
mod tests;

use super::AccountInformationStorage;
use crate::types::{StorableAccount, StoredAccountMode};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::{File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
};
#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};

/// Represents the version of the mnemonic file.
#[allow(unused)] // Will be used when migration is required
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum OnDiskAccountStorageVersion {
    V1,
}

impl OnDiskAccountStorageVersion {
    /// Returns the latest version of the mnemonic file.
    #[allow(unused)] // Will be used when migration is required
    pub fn latest() -> Self {
        OnDiskAccountStorageVersion::V1
    }
}

impl fmt::Display for OnDiskAccountStorageVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            OnDiskAccountStorageVersion::V1 => "v1",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
#[serde(rename_all = "snake_case")]
enum StoredAccountsExt {
    V1(v1::StoredAccounts),
}

impl StoredAccountsExt {
    fn insert_account(&mut self, account: StorableAccount) {
        match self {
            StoredAccountsExt::V1(v1) => {
                v1.insert_account(account);
            }
        }
    }

    fn remove_account(&mut self, stored_account_mode: Option<StoredAccountMode>) -> bool {
        match self {
            StoredAccountsExt::V1(v1) => v1.remove_account(stored_account_mode),
        }
    }
}

impl Default for StoredAccountsExt {
    fn default() -> Self {
        StoredAccountsExt::V1(v1::StoredAccounts::default())
    }
}

impl TryFrom<StoredAccountsExt> for Vec<StorableAccount> {
    type Error = OnDiskMnemonicStorageError;

    fn try_from(value: StoredAccountsExt) -> Result<Self, Self::Error> {
        match value {
            StoredAccountsExt::V1(accounts) => accounts.try_into(),
        }
    }
}

pub struct OnDiskAccountStorage {
    path: PathBuf,
}

impl OnDiskAccountStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    async fn load_stored_accounts_ext(
        &self,
    ) -> Result<StoredAccountsExt, OnDiskMnemonicStorageError> {
        tracing::debug!("Loading accounts from: {}", self.path.display());

        // Make sure that the file has permissions set to 600 (rw------)
        #[cfg(unix)]
        {
            let permissions = Permissions::from_mode(0o600);
            if let Err(e) = tokio::fs::set_permissions(&self.path, permissions).await
                && e.kind() != std::io::ErrorKind::NotFound
            {
                return Err(OnDiskMnemonicStorageError::FileOpenError(e));
            }
        }

        // If the files does not exist, then it's not an error.
        let mut file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(StoredAccountsExt::default());
            }
            Err(e) => return Err(OnDiskMnemonicStorageError::FileOpenError(e)),
        };

        // First try loading the versioned mnemonic file, before falling back to the legacy one.
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(OnDiskMnemonicStorageError::FileOpenError)?;

        match serde_json::from_slice::<StoredAccountsExt>(&bytes) {
            Ok(stored_accounts) => Ok(stored_accounts),
            Err(_) => {
                let legacy_account: legacy::StoredAccount = serde_json::from_slice(&bytes)
                    .map_err(OnDiskMnemonicStorageError::ReadError)?;
                let stored_accounts: v1::StoredAccounts = legacy_account.try_into()?;
                let stored_accounts_ext: StoredAccountsExt = StoredAccountsExt::V1(stored_accounts);
                Ok(stored_accounts_ext)
            }
        }
    }

    async fn save_stored_accounts_ext(
        &self,
        stored_accounts: &StoredAccountsExt,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        tracing::info!("Storing accounts to: {}", self.path.display());

        // Create parent directories
        tracing::trace!("Creating parent directories for: {}", self.path.display());
        if let Some(parent) = self.path.parent() {
            tracing::trace!("Creating parent directory: {}", parent.display());
            tokio::fs::create_dir_all(parent).await.map_err(|err| {
                OnDiskMnemonicStorageError::FileCreateError {
                    path: parent.to_path_buf(),
                    source: err,
                }
            })?;

            #[cfg(unix)]
            {
                // Set directory permissions to 700 (rwx------)
                tracing::trace!("Set directory permissions to 700 (rwx------)");
                let permissions = Permissions::from_mode(0o700);
                tokio::fs::set_permissions(parent, permissions)
                    .await
                    .map_err(|source| OnDiskMnemonicStorageError::FileCreateError {
                        path: parent.to_path_buf(),
                        source,
                    })?;
            }

            // TODO: same for windows
        }

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|err| OnDiskMnemonicStorageError::FileCreateError {
                path: self.path.clone(),
                source: err,
            })?;

        serde_json::to_writer(file, &stored_accounts)
            .map_err(OnDiskMnemonicStorageError::WriteError)?;

        #[cfg(unix)]
        {
            // Set directory permissions to 600 (rw------)
            let permissions = Permissions::from_mode(0o600);
            tokio::fs::set_permissions(self.path.clone(), permissions)
                .await
                .map_err(|source| OnDiskMnemonicStorageError::FileCreateError {
                    path: self.path.clone(),
                    source,
                })?;
        }

        // TODO: same for windows

        Ok(())
    }
}

#[async_trait::async_trait]
impl AccountInformationStorage for OnDiskAccountStorage {
    type StorageError = OnDiskMnemonicStorageError;

    async fn load_accounts(&self) -> Result<Vec<StorableAccount>, OnDiskMnemonicStorageError> {
        let stored_accounts_ext = self.load_stored_accounts_ext().await?;
        stored_accounts_ext.try_into()
    }

    async fn store_account(
        &self,
        account: StorableAccount,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        // If the currently stored accounts file is corrupted then all its data will be lost.
        // There is nothing we can do about that.
        let mut stored_accounts_ext = self.load_stored_accounts_ext().await.unwrap_or_else(|err| {
            tracing::error!(
                "Failed to load stored accounts while storing a new account. Possible data loss: {err}"
            );
            StoredAccountsExt::default()
        });

        stored_accounts_ext.insert_account(account);

        self.save_stored_accounts_ext(&stored_accounts_ext).await
    }

    async fn remove_account(
        &self,
        stored_account_mode: Option<StoredAccountMode>,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        // If the currently stored accounts file is corrupted then all its data will be lost.
        // There is nothing we can do about that.
        let mut stored_accounts = self.load_stored_accounts_ext().await.unwrap_or_else(|err| {
            tracing::error!(
                "Failed to load stored accounts while attempting to remove an account. Possible data loss: {err}"
            );
            StoredAccountsExt::default()
        });

        if stored_accounts.remove_account(stored_account_mode) {
            self.save_stored_accounts_ext(&stored_accounts).await
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OnDiskMnemonicStorageError {
    #[error("mnemonic already stored")]
    MnemonicAlreadyStored { path: PathBuf },

    #[error("failed to create file")]
    FileCreateError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to open file")]
    FileOpenError(#[source] std::io::Error),

    #[error("failed to read mnemonic from file")]
    ReadError(#[source] serde_json::Error),

    #[error("failed to write mnemonic to file")]
    WriteError(#[source] serde_json::Error),

    #[error("failed to remove mnemonic file")]
    RemoveError(#[source] std::io::Error),
}
