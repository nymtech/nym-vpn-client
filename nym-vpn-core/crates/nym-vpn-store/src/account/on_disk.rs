// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
};

use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use super::{AccountInformationStorage, StoredDeviceState};
use crate::store_ciphers::{StoreCipher, StoreCipherError};

#[derive(Debug, thiserror::Error)]
pub enum OnDiskAccountStorageError {
    #[error("failed to {ctx}: {path}")]
    Io {
        ctx: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serialize device state")]
    Serialize(#[source] serde_json::Error),

    #[error("failed to deserialize device state")]
    Deserialize(#[source] serde_json::Error),

    #[error("failed to encrypt data")]
    Encrypt(#[source] StoreCipherError),

    #[error("failed to decrypt data")]
    Decrypt(#[source] StoreCipherError),
}

pub type Result<T, E = OnDiskAccountStorageError> = std::result::Result<T, E>;

pub struct OnDiskAccountStorage {
    path: PathBuf,
    cipher: Box<dyn StoreCipher>,
}

impl OnDiskAccountStorage {
    pub fn new(path: impl AsRef<Path>, cipher: impl StoreCipher + 'static) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            cipher: Box::new(cipher),
        }
    }

    async fn create_parent_dirs(&self) -> Result<()> {
        let Some(parent) = self.path.parent() else {
            return Ok(());
        };

        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| OnDiskAccountStorageError::Io {
                ctx: "create directory",
                path: parent.to_path_buf(),
                source: err,
            })?;

        #[cfg(unix)]
        tokio::fs::set_permissions(parent, Permissions::from_mode(0o700))
            .await
            .map_err(|err| OnDiskAccountStorageError::Io {
                ctx: "set parent dir permissions",
                path: parent.to_path_buf(),
                source: err,
            })?;

        Ok(())
    }

    async fn write_atomic(&self, ciphertext: &[u8]) -> Result<()> {
        let temp_path = self.path.with_added_extension("tmp");
        let temp_io_error = |ctx, source| OnDiskAccountStorageError::Io {
            ctx,
            path: temp_path.clone(),
            source,
        };

        {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temp_path)
                .await
                .map_err(|err| temp_io_error("create file", err))?;

            #[cfg(unix)]
            file.set_permissions(Permissions::from_mode(0o600))
                .await
                .map_err(|err| temp_io_error("set file permissions", err))?;

            file.write_all(ciphertext)
                .await
                .map_err(|err| temp_io_error("write file", err))?;
            file.sync_all()
                .await
                .map_err(|err| temp_io_error("sync file", err))?;
        }

        tokio::fs::rename(&temp_path, &self.path)
            .await
            .map_err(|err| temp_io_error("rename file", err))
    }

    fn io_error(&self, ctx: &'static str, source: std::io::Error) -> OnDiskAccountStorageError {
        OnDiskAccountStorageError::Io {
            ctx,
            path: self.path.clone(),
            source,
        }
    }
}

#[async_trait::async_trait]
impl AccountInformationStorage for OnDiskAccountStorage {
    type StorageError = OnDiskAccountStorageError;

    async fn load_account(&self) -> Result<Option<StoredDeviceState>> {
        match tokio::fs::read(&self.path).await {
            Ok(ciphertext) => {
                let plaintext = self
                    .cipher
                    .decrypt(&ciphertext)
                    .await
                    .map_err(OnDiskAccountStorageError::Decrypt)?;

                serde_json::from_slice(&plaintext)
                    .map(Some)
                    .map_err(OnDiskAccountStorageError::Deserialize)
            }
            Err(err) if err.kind() == IoErrorKind::NotFound => Ok(None),
            Err(err) => Err(self.io_error("read file", err)),
        }
    }

    async fn store_account(&self, state: StoredDeviceState) -> Result<()> {
        tracing::debug!("Storing device state to: {}", self.path.display());

        self.create_parent_dirs().await?;

        let plaintext = serde_json::to_vec(&state).map_err(OnDiskAccountStorageError::Serialize)?;
        let ciphertext = self
            .cipher
            .encrypt(&plaintext)
            .await
            .map_err(OnDiskAccountStorageError::Encrypt)?;

        self.write_atomic(&ciphertext).await
    }

    async fn remove_account(&self) -> Result<()> {
        match tokio::fs::remove_file(&self.path).await {
            Err(err) if err.kind() != IoErrorKind::NotFound => {
                Err(self.io_error("remove file", err))
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use nym_credentials_interface::KeyPairUser;
    use nym_vpn_lib_types::StoredAccountMode;

    use super::*;
    use crate::{account::StoredAccountIdentity, store_ciphers::noop_cipher::NoopStoreCipher};

    fn logged_in(mode: StoredAccountMode) -> StoredDeviceState {
        StoredDeviceState {
            account_identity: StoredAccountIdentity {
                id: "juno1cma4czt2jnydvrvz3lrc9jvcmhpjxtds95s3c6".to_owned(),
                mode: mode,
                pub_key: "".to_owned(),
                signature_base64: "".to_owned(),
            },
            ecash_keypair: KeyPairUser::new(),
            summary: None,
        }
    }

    fn storage(path: &Path) -> OnDiskAccountStorage {
        OnDiskAccountStorage::new(path, NoopStoreCipher)
    }

    #[tokio::test]
    async fn load_returns_none_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("account.bin"));

        assert_eq!(storage.load_account().await.unwrap(), None);
    }

    #[tokio::test]
    async fn store_and_load() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("nested/account.bin"));

        let state = logged_in(StoredAccountMode::Api);
        storage.store_account(state.clone()).await.unwrap();

        assert_eq!(storage.load_account().await.unwrap(), Some(state));
    }

    #[tokio::test]
    async fn store_overwrites_previous_state() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("account.bin");
        let storage = storage(&path);

        storage
            .store_account(logged_in(StoredAccountMode::Api))
            .await
            .unwrap();
        let state = logged_in(StoredAccountMode::Decentralised);
        storage.store_account(state.clone()).await.unwrap();

        assert_eq!(storage.load_account().await.unwrap(), Some(state));
        assert!(!path.with_added_extension("tmp").exists());
    }

    #[tokio::test]
    async fn remove_deletes_state() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("account.bin");
        let storage = storage(&path);

        storage
            .store_account(logged_in(StoredAccountMode::Api))
            .await
            .unwrap();
        storage.remove_account().await.unwrap();

        assert!(!path.exists());
        assert_eq!(storage.load_account().await.unwrap(), None);
    }

    #[tokio::test]
    async fn remove_succeeds_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("account.bin"));

        storage.remove_account().await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stored_file_is_private() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("account.bin");
        let storage = storage(&path);

        storage
            .store_account(logged_in(StoredAccountMode::Api))
            .await
            .unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
