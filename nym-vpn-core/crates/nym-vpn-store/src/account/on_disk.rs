// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AccountInformationStorage, StoredAccount, StoredAccounts};
use crate::types::{StorableAccount, StoredAccountMode};
#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
};

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

pub struct OnDiskAccountStorage {
    path: PathBuf,
}

impl OnDiskAccountStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    async fn load_stored_accounts(&self) -> Result<StoredAccounts, OnDiskMnemonicStorageError> {
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
                return Ok(StoredAccounts(std::collections::HashMap::new()));
            }
            Err(e) => return Err(OnDiskMnemonicStorageError::FileOpenError(e)),
        };

        // First try loading the dictionary of accounts, and if that fails try to load a single account
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(OnDiskMnemonicStorageError::FileOpenError)?;

        match serde_json::from_slice::<StoredAccounts>(&bytes) {
            Ok(stored_accounts) => Ok(stored_accounts),
            Err(_) => {
                let stored_account: StoredAccount = serde_json::from_slice(&bytes)
                    .map_err(OnDiskMnemonicStorageError::ReadError)?;
                Ok(StoredAccounts::with_account(stored_account))
            }
        }
    }

    async fn save_stored_accounts(
        &self,
        stored_accounts: &StoredAccounts,
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
        let stored_accounts = self.load_stored_accounts().await?;
        Ok(stored_accounts
            .0
            .into_values()
            .map(|account| account.into())
            .collect())
    }

    async fn store_account(
        &self,
        account: StorableAccount,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        // If the currently stored accounts file is corrupted then all its data will be lost.
        // There is nothing we can do about that.
        let mut stored_accounts = self.load_stored_accounts().await.unwrap_or_else(|err| {
            tracing::error!(
                "Failed to load stored accounts while storing a new account. Possible data loss: {err}"
            );
            StoredAccounts::default()
        });

        let name = "default".to_string();
        let nonce = 0;
        let stored_account = StoredAccount {
            name,
            mnemonic: account.mnemonic,
            mode: account.mode,
            nonce,
        };

        stored_accounts
            .0
            .insert(stored_account.mode, stored_account);

        self.save_stored_accounts(&stored_accounts).await
    }

    async fn remove_account(
        &self,
        stored_account_mode: Option<StoredAccountMode>,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        // If the currently stored accounts file is corrupted then all its data will be lost.
        // There is nothing we can do about that.
        let mut stored_accounts = self.load_stored_accounts().await.unwrap_or_else(|err| {
            tracing::error!(
                "Failed to load stored accounts while attempting to remove an account. Possible data loss: {err}"
            );
            StoredAccounts::default()
        });

        if let Some(stored_account_mode) = stored_account_mode {
            if stored_accounts.0.contains_key(&stored_account_mode) {
                stored_accounts.0.remove(&stored_account_mode);
            }
        } else {
            stored_accounts.0.clear();
        }

        // Always write back to disk, even if nothing changed.
        self.save_stored_accounts(&stored_accounts).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::test_fixtures::account_fixture;

    #[tokio::test]
    async fn store_account() {
        let account = account_fixture();

        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");

        let account_storage = OnDiskAccountStorage::new(path);
        account_storage
            .store_account(account.clone())
            .await
            .unwrap();

        let stored_accounts = account_storage.load_accounts().await.unwrap();
        assert_eq!(vec![account], stored_accounts);
    }

    #[tokio::test]
    async fn store_twice_overwrites_for_same_mode() {
        let account = account_fixture();

        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let account_storage = OnDiskAccountStorage::new(path);

        account_storage
            .store_account(account.clone())
            .await
            .unwrap();
        account_storage
            .store_account(account.clone())
            .await
            .unwrap();

        let stored_accounts = account_storage.load_accounts().await.unwrap();
        assert_eq!(vec![account], stored_accounts);
    }

    #[tokio::test]
    async fn load_returns_empty_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let account_storage = OnDiskAccountStorage::new(path);

        let result = account_storage.load_accounts().await;
        assert!(matches!(result, Ok(v) if v.is_empty()));
    }

    #[tokio::test]
    async fn load_fails_if_file_contains_invalid_json() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let account_storage = OnDiskAccountStorage::new(path.clone());

        // Write invalid JSON so serde_json definitely errors
        std::fs::write(&path, b"not json").unwrap();

        let result = account_storage.load_accounts().await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::ReadError(_))
        ));
    }

    #[tokio::test]
    async fn load_of_legacy_single_account_json_still_works() -> anyhow::Result<()> {
        let account = account_fixture();

        // Legacy shape supported: a single `StoredAccount` JSON object (not the map).
        let legacy_single = StoredAccount {
            name: "foomp".to_string(),
            mnemonic: account.mnemonic.clone(),
            mode: account.mode,
            nonce: 0,
        };

        let tempdir = tempfile::tempdir()?;
        let path = tempdir.path().join("test.txt");

        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        serde_json::to_writer(file, &legacy_single)?;

        let account_storage = OnDiskAccountStorage::new(path);
        let loaded = account_storage.load_accounts().await?;
        assert_eq!(vec![account], loaded);

        Ok(())
    }
}
