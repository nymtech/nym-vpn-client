// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
};

use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use zeroize::Zeroizing;

use super::{MnemonicStorage, StoredMnemonic};
use crate::store_ciphers::{StoreCipher, StoreCipherError};

#[derive(Debug, thiserror::Error)]
pub enum OnDiskMnemonicStorageError {
    #[error("mnemonic already stored")]
    MnemonicAlreadyStored { path: PathBuf },

    #[error("failed to {ctx}: {path}")]
    Io {
        ctx: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serialize mnemonic")]
    Serialize(#[source] serde_json::Error),

    #[error("failed to deserialize mnemonic")]
    Deserialize(#[source] serde_json::Error),

    #[error("failed to encrypt data")]
    Encrypt(#[source] StoreCipherError),

    #[error("failed to decrypt data")]
    Decrypt(#[source] StoreCipherError),
}

pub type Result<T, E = OnDiskMnemonicStorageError> = std::result::Result<T, E>;

pub struct OnDiskMnemonicStorage {
    path: PathBuf,
    cipher: Box<dyn StoreCipher>,
}

impl OnDiskMnemonicStorage {
    pub fn new(path: impl AsRef<Path>, cipher: impl StoreCipher + 'static) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            cipher: Box::new(cipher),
        }
    }

    async fn check_mnemonic_exists(&self) -> Result<()> {
        tokio::fs::try_exists(&self.path)
            .await
            .map_err(|err| self.io_error("check if file exists", err))
            .and_then(|mnemonic_exists| {
                if mnemonic_exists {
                    Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored {
                        path: self.path.clone(),
                    })
                } else {
                    Ok(())
                }
            })
    }

    async fn create_parent_dirs(&self) -> Result<()> {
        let Some(parent) = self.path.parent() else {
            return Ok(());
        };

        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| OnDiskMnemonicStorageError::Io {
                ctx: "create directory",
                path: parent.to_path_buf(),
                source: err,
            })?;

        #[cfg(unix)]
        tokio::fs::set_permissions(parent, Permissions::from_mode(0o700))
            .await
            .map_err(|err| OnDiskMnemonicStorageError::Io {
                ctx: "set parent dir permissions",
                path: parent.to_path_buf(),
                source: err,
            })?;

        Ok(())
    }

    async fn write_to_file(&self, ciphertext: &[u8]) -> Result<()> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)
            .await
            .map_err(|err| {
                if err.kind() == IoErrorKind::AlreadyExists {
                    OnDiskMnemonicStorageError::MnemonicAlreadyStored {
                        path: self.path.clone(),
                    }
                } else {
                    self.io_error("create file", err)
                }
            })?;

        #[cfg(unix)]
        file.set_permissions(Permissions::from_mode(0o600))
            .await
            .map_err(|err| self.io_error("set file permissions", err))?;

        file.write_all(ciphertext)
            .await
            .map_err(|err| self.io_error("write file", err))
    }

    fn io_error(&self, ctx: &'static str, source: std::io::Error) -> OnDiskMnemonicStorageError {
        OnDiskMnemonicStorageError::Io {
            ctx,
            path: self.path.clone(),
            source,
        }
    }
}

#[async_trait::async_trait]
impl MnemonicStorage for OnDiskMnemonicStorage {
    type StorageError = OnDiskMnemonicStorageError;

    async fn load_mnemonic(&self) -> Result<Option<StoredMnemonic>> {
        match tokio::fs::read(&self.path).await {
            Ok(ciphertext) => {
                let plaintext = self
                    .cipher
                    .decrypt(&ciphertext)
                    .await
                    .map(Zeroizing::new)
                    .map_err(OnDiskMnemonicStorageError::Decrypt)?;

                serde_json::from_slice::<StoredMnemonic>(&plaintext)
                    .map(Some)
                    .map_err(OnDiskMnemonicStorageError::Deserialize)
            }
            Err(err) if err.kind() == IoErrorKind::NotFound => Ok(None),
            Err(err) => Err(self.io_error("read file", err)),
        }
    }

    async fn store_mnemonic(&self, mnemonic: StoredMnemonic) -> Result<()> {
        tracing::info!("Storing mnemonic to: {}", self.path.display());

        self.check_mnemonic_exists().await?;
        self.create_parent_dirs().await?;

        let plaintext = serde_json::to_vec(&mnemonic)
            .map(Zeroizing::new)
            .map_err(OnDiskMnemonicStorageError::Serialize)?;
        let ciphertext = self
            .cipher
            .encrypt(&plaintext)
            .await
            .map_err(OnDiskMnemonicStorageError::Encrypt)?;

        self.write_to_file(&ciphertext).await
    }

    async fn remove_mnemonic(&self) -> Result<()> {
        tokio::fs::remove_file(&self.path).await.or_else(|err| {
            if err.kind() == IoErrorKind::NotFound {
                Ok(())
            } else {
                Err(self.io_error("remove file", err))
            }
        })
    }

    async fn is_mnemonic_stored(&self) -> Result<bool> {
        self.load_mnemonic().await.map(|v| v.is_some())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;
    use crate::{
        mnemonic::test_fixtures::{mnemonic_fixture, stored_mnemonic_fixture},
        store_ciphers::noop_cipher::NoopStoreCipher,
    };

    fn storage(path: &Path) -> OnDiskMnemonicStorage {
        OnDiskMnemonicStorage::new(path, NoopStoreCipher)
    }

    #[tokio::test]
    async fn store_and_load_mnemonic() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("nested/mnemonic.json"));

        storage
            .store_mnemonic(stored_mnemonic_fixture())
            .await
            .unwrap();

        let loaded = storage.load_mnemonic().await.unwrap();
        assert_matches!(loaded, Some(stored) if stored.mnemonic == mnemonic_fixture());
    }

    #[tokio::test]
    async fn store_twice_fails() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("mnemonic.json"));

        storage
            .store_mnemonic(stored_mnemonic_fixture())
            .await
            .unwrap();

        let result = storage.store_mnemonic(stored_mnemonic_fixture()).await;
        assert_matches!(
            result,
            Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored { .. })
        );
    }

    #[tokio::test]
    async fn load_returns_none_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("mnemonic.json"));

        assert_matches!(storage.load_mnemonic().await, Ok(None));
    }

    #[tokio::test]
    async fn load_fails_if_bogus_empty_file_exists() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("mnemonic.json");
        tokio::fs::File::create(&path).await.unwrap();

        assert!(storage(&path).load_mnemonic().await.is_err());
    }

    #[tokio::test]
    async fn remove_mnemonic() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("mnemonic.json");
        let storage = storage(&path);

        storage
            .store_mnemonic(stored_mnemonic_fixture())
            .await
            .unwrap();
        storage.remove_mnemonic().await.unwrap();

        assert!(!path.exists());
        assert!(!storage.is_mnemonic_stored().await.unwrap());
    }

    #[tokio::test]
    async fn remove_succeeds_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("mnemonic.json"));

        storage.remove_mnemonic().await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stored_file_is_private() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("mnemonic.json");

        storage(&path)
            .store_mnemonic(stored_mnemonic_fixture())
            .await
            .unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
