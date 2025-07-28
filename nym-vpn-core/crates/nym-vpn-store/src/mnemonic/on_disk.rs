// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

use super::{MnemonicStorage, StoredMnemonic};

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

pub struct OnDiskMnemonicStorage {
    path: PathBuf,
}

impl OnDiskMnemonicStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait::async_trait]
impl MnemonicStorage for OnDiskMnemonicStorage {
    type StorageError = OnDiskMnemonicStorageError;

    async fn store_mnemonic(
        &self,
        mnemonic: bip39::Mnemonic,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        let name = "default".to_string();
        let nonce = 0;
        let stored_mnemonic = StoredMnemonic {
            name,
            mnemonic,
            nonce,
        };

        tracing::info!("Storing mnemonic to: {}", self.path.display());

        // Error if the file already exists
        if self.path.exists() {
            return Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored {
                path: self.path.clone(),
            });
        }

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

        // Another layer of defense, only create the file if it doesn't already exist
        tracing::debug!("Only creating the file if it doesn't already exist");
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)
            .map_err(|err| OnDiskMnemonicStorageError::FileCreateError {
                path: self.path.clone(),
                source: err,
            })?;

        serde_json::to_writer(file, &stored_mnemonic)
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

    async fn load_mnemonic(&self) -> Result<Option<bip39::Mnemonic>, OnDiskMnemonicStorageError> {
        tracing::debug!("Opening: {}", self.path.display());

        // Make sure that the file has permissions set to 600 (rw------)
        #[cfg(unix)]
        {
            let permissions = Permissions::from_mode(0o600);
            if let Err(e) = tokio::fs::set_permissions(&self.path, permissions).await {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(OnDiskMnemonicStorageError::FileOpenError(e));
                }
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
            .map(|s: StoredMnemonic| Some(s.mnemonic.clone()))
    }

    async fn remove_mnemonic(&self) -> Result<(), OnDiskMnemonicStorageError> {
        if !self.path.exists() {
            return Ok(());
        }
        tokio::fs::remove_file(&self.path)
            .await
            .map_err(OnDiskMnemonicStorageError::RemoveError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn store_mnemonic() {
        let mnemonic = bip39::Mnemonic::generate_in(bip39::Language::English, 12).unwrap();
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let mnemonic_storage = OnDiskMnemonicStorage::new(path.clone());
        mnemonic_storage
            .store_mnemonic(mnemonic.clone())
            .await
            .unwrap();

        let stored_mnemonic = mnemonic_storage.load_mnemonic().await.unwrap();
        assert_eq!(Some(mnemonic), stored_mnemonic);
    }

    #[tokio::test]
    async fn store_twice_fails() {
        let mnemonic = bip39::Mnemonic::generate_in(bip39::Language::English, 12).unwrap();
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let mnemonic_storage = OnDiskMnemonicStorage::new(path.clone());
        mnemonic_storage
            .store_mnemonic(mnemonic.clone())
            .await
            .unwrap();

        let result = mnemonic_storage.store_mnemonic(mnemonic).await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored { .. })
        ));
    }

    #[tokio::test]
    async fn load_return_none_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let mnemonic_storage = OnDiskMnemonicStorage::new(path.clone());
        let result = mnemonic_storage.load_mnemonic().await;
        assert!(matches!(result, Ok(None)));
    }

    #[tokio::test]
    async fn load_fails_if_no_mnemonic_stored() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let mnemonic_storage = OnDiskMnemonicStorage::new(path.clone());
        let _ = File::create(&path).unwrap();
        let result = mnemonic_storage.load_mnemonic().await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::ReadError(_))
        ));
    }
}
