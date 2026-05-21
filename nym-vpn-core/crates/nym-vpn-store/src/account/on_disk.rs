// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AccountInformationStorage, StoredAccount};
use crate::types::StorableAccount;
#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    fs::{File, OpenOptions},
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

    async fn write_to(
        &self,
        path: &Path,
        account: &StoredAccount,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        // Create parent directories
        tracing::trace!("Creating parent directories for: {}", path.display());
        if let Some(parent) = path.parent() {
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

        // Create the file
        tracing::debug!("Creating file: {}", path.display());
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|err| OnDiskMnemonicStorageError::FileCreateError {
                path: path.to_path_buf(),
                source: err,
            })?;

        serde_json::to_writer(file, account)
            .map_err(OnDiskMnemonicStorageError::WriteError)?;

        #[cfg(unix)]
        {
            // Set file permissions to 600 (rw------)
            let permissions = Permissions::from_mode(0o600);
            tokio::fs::set_permissions(path, permissions)
                .await
                .map_err(|source| OnDiskMnemonicStorageError::FileCreateError {
                    path: path.to_path_buf(),
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

    async fn store_account(
        &self,
        account: StorableAccount,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        let name = "default".to_string();
        let nonce = 0;
        let stored_account = StoredAccount {
            name,
            mnemonic: account.mnemonic,
            mode: account.mode,
            nonce,
            is_locally_generated: account.is_locally_generated,
            is_registered_with_api: account.is_registered_with_api,
            is_backup_confirmed: account.is_backup_confirmed,
        };

        tracing::info!("Storing mnemonic to: {}", self.path.display());

        // Error if the file already exists
        if self.path.exists() {
            return Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored {
                path: self.path.clone(),
            });
        }

        self.write_to(&self.path, &stored_account).await
    }

    async fn load_account(&self) -> Result<Option<StorableAccount>, OnDiskMnemonicStorageError> {
        tracing::debug!("Opening: {}", self.path.display());

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
        // We still that checks, for non-unix at least
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(OnDiskMnemonicStorageError::FileOpenError(e)),
        };

        serde_json::from_reader(file)
            .map_err(OnDiskMnemonicStorageError::ReadError)
            .map(|s: StoredAccount| Some(s.into()))
    }

    async fn remove_account(&self) -> Result<(), OnDiskMnemonicStorageError> {
        if !self.path.exists() {
            return Ok(());
        }
        tokio::fs::remove_file(&self.path)
            .await
            .map_err(OnDiskMnemonicStorageError::RemoveError)
    }

    async fn update_account<F>(&self, f: F) -> Result<(), OnDiskMnemonicStorageError>
    where
        F: FnOnce(&mut StorableAccount) + Send,
    {
        let mut account = self
            .load_account()
            .await?
            .ok_or_else(|| {
                OnDiskMnemonicStorageError::FileOpenError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "no account stored",
                ))
            })?;
        f(&mut account);

        // Atomic rename: write to temp file, then rename to original.
        // This avoids a crash window where the file doesn't exist.
        let stored_account = StoredAccount {
            name: "default".to_string(),
            mnemonic: account.mnemonic,
            mode: account.mode,
            nonce: 0,
            is_locally_generated: account.is_locally_generated,
            is_registered_with_api: account.is_registered_with_api,
            is_backup_confirmed: account.is_backup_confirmed,
        };

        let tmp_path = self.path.with_extension("tmp");

        // Write to temp file
        if let Err(e) = self.write_to(&tmp_path, &stored_account).await {
            // Best-effort cleanup
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(e);
        }

        // Atomically rename temp file to original
        tokio::fs::rename(&tmp_path, &self.path)
            .await
            .map_err(|source| {
                // Best-effort cleanup on rename failure
                drop(tokio::spawn(async move {
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                }));
                OnDiskMnemonicStorageError::FileCreateError {
                    path: self.path.clone(),
                    source,
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        account::{
            Nonce,
            test_fixtures::{account_fixture, mnemonic_fixture},
        },
        types::StoredAccountMode,
    };
    use bip39::Mnemonic;
    use serde::{Deserialize, Serialize};

    #[tokio::test]
    async fn store_account() {
        let account = account_fixture();

        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");

        let account_storage = OnDiskAccountStorage::new(path.clone());
        account_storage
            .store_account(account.clone())
            .await
            .unwrap();

        let stored_account = account_storage.load_account().await.unwrap();
        assert_eq!(Some(account), stored_account);
    }

    #[tokio::test]
    async fn store_twice_fails() {
        let account = account_fixture();

        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let account_storage = OnDiskAccountStorage::new(path.clone());
        account_storage
            .store_account(account.clone())
            .await
            .unwrap();

        let result = account_storage.store_account(account).await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored { .. })
        ));
    }

    #[tokio::test]
    async fn load_return_none_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let account_storage = OnDiskAccountStorage::new(path.clone());
        let result = account_storage.load_account().await;
        assert!(matches!(result, Ok(None)));
    }

    #[tokio::test]
    async fn load_fails_if_no_mnemonic_stored() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let account_storage = OnDiskAccountStorage::new(path.clone());
        let _ = File::create(&path).unwrap();
        let result = account_storage.load_account().await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::ReadError(_))
        ));
    }

    #[tokio::test]
    async fn load_of_legacy_mnemonics_still_works() -> anyhow::Result<()> {
        #[derive(Serialize, Deserialize)]
        struct LegacyStoredMnemonic {
            name: String,
            mnemonic: Mnemonic,
            nonce: Nonce,
        }

        let legacy = LegacyStoredMnemonic {
            name: "foomp".to_string(),
            mnemonic: mnemonic_fixture(),
            nonce: 0,
        };

        let tempdir = tempfile::tempdir()?;
        let path = tempdir.path().join("test.txt");

        // save legacy data
        tokio::fs::create_dir_all(tempdir.path()).await?;
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        serde_json::to_writer(file, &legacy)?;

        let expected = StorableAccount {
            mnemonic: mnemonic_fixture(),
            mode: StoredAccountMode::Api,
            is_locally_generated: false,
            is_registered_with_api: false,
            is_backup_confirmed: false,
        };

        let account_storage = OnDiskAccountStorage::new(path.clone());
        let loaded = account_storage.load_account().await?;
        assert_eq!(Some(expected), loaded);

        Ok(())
    }

    #[tokio::test]
    async fn update_account_sets_flags() {
        let account = account_fixture();
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let storage = OnDiskAccountStorage::new(path);
        storage.store_account(account.clone()).await.unwrap();

        storage
            .update_account(|a: &mut StorableAccount| {
                a.is_registered_with_api = true;
                a.is_backup_confirmed = true;
            })
            .await
            .unwrap();

        let loaded = storage.load_account().await.unwrap().unwrap();
        assert!(loaded.is_registered_with_api);
        assert!(loaded.is_backup_confirmed);
        assert!(!loaded.is_locally_generated);
    }

    #[tokio::test]
    async fn on_disk_round_trips_new_flags() {
        let mnemonic = mnemonic_fixture();
        let account = StorableAccount {
            mnemonic: mnemonic.clone(),
            mode: StoredAccountMode::Api,
            is_locally_generated: true,
            is_registered_with_api: false,
            is_backup_confirmed: true,
        };
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let storage = OnDiskAccountStorage::new(path);
        storage.store_account(account.clone()).await.unwrap();

        let loaded = storage.load_account().await.unwrap().unwrap();
        assert!(loaded.is_locally_generated);
        assert!(!loaded.is_registered_with_api);
        assert!(loaded.is_backup_confirmed);
        assert_eq!(loaded.mnemonic, mnemonic);
    }

    #[tokio::test]
    async fn update_account_no_temp_file_leftover() {
        let account = account_fixture();
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let storage = OnDiskAccountStorage::new(path.clone());
        storage.store_account(account.clone()).await.unwrap();

        // Update the account with flag-flipping mutation
        storage
            .update_account(|a: &mut StorableAccount| {
                a.is_registered_with_api = true;
                a.is_backup_confirmed = true;
            })
            .await
            .unwrap();

        // Assert the original file still exists
        assert!(path.exists(), "Original file should exist after update");

        // Assert the updated flags
        let loaded = storage.load_account().await.unwrap().unwrap();
        assert!(loaded.is_registered_with_api);
        assert!(loaded.is_backup_confirmed);

        // Assert no temp file is left behind
        let tmp_path = path.with_extension("tmp");
        assert!(
            !tmp_path.exists(),
            "Temporary file should not exist after successful update"
        );
    }
}
