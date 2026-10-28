// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
};

use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use super::{DeviceStateStorage, StoredDeviceState};
use crate::store_ciphers::{StoreCipher, StoreCipherError};

#[derive(Debug, thiserror::Error)]
pub enum OnDiskDeviceStateStorageError {
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

pub type Result<T, E = OnDiskDeviceStateStorageError> = std::result::Result<T, E>;

pub struct OnDiskDeviceStateStorage {
    path: PathBuf,
    cipher: Box<dyn StoreCipher>,
}

impl OnDiskDeviceStateStorage {
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

        tokio::fs::create_dir_all(parent).await.map_err(|err| {
            OnDiskDeviceStateStorageError::Io {
                ctx: "create directory",
                path: parent.to_path_buf(),
                source: err,
            }
        })?;

        #[cfg(unix)]
        tokio::fs::set_permissions(parent, Permissions::from_mode(0o700))
            .await
            .map_err(|err| OnDiskDeviceStateStorageError::Io {
                ctx: "set parent dir permissions",
                path: parent.to_path_buf(),
                source: err,
            })?;

        Ok(())
    }

    async fn write_atomic(&self, ciphertext: &[u8]) -> Result<()> {
        let temp_path = self.path.with_added_extension("tmp");
        let temp_io_error = |ctx, source| OnDiskDeviceStateStorageError::Io {
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

    fn io_error(&self, ctx: &'static str, source: std::io::Error) -> OnDiskDeviceStateStorageError {
        OnDiskDeviceStateStorageError::Io {
            ctx,
            path: self.path.clone(),
            source,
        }
    }
}

#[async_trait::async_trait]
impl DeviceStateStorage for OnDiskDeviceStateStorage {
    type StorageError = OnDiskDeviceStateStorageError;

    async fn load_device_state(&self) -> Result<StoredDeviceState> {
        match tokio::fs::read(&self.path).await {
            Ok(ciphertext) => {
                let plaintext = self
                    .cipher
                    .decrypt(&ciphertext)
                    .await
                    .map_err(OnDiskDeviceStateStorageError::Decrypt)?;

                serde_json::from_slice(&plaintext)
                    .map_err(OnDiskDeviceStateStorageError::Deserialize)
            }
            Err(err) if err.kind() == IoErrorKind::NotFound => Ok(StoredDeviceState::LoggedOut),
            Err(err) => Err(self.io_error("read file", err)),
        }
    }

    async fn store_device_state(&self, state: StoredDeviceState) -> Result<()> {
        tracing::debug!("Storing device state to: {}", self.path.display());

        self.create_parent_dirs().await?;

        let plaintext =
            serde_json::to_vec(&state).map_err(OnDiskDeviceStateStorageError::Serialize)?;
        let ciphertext = self
            .cipher
            .encrypt(&plaintext)
            .await
            .map_err(OnDiskDeviceStateStorageError::Encrypt)?;

        self.write_atomic(&ciphertext).await
    }

    async fn remove_device_state(&self) -> Result<()> {
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
    use nym_vpn_lib_types::StoredAccountMode;

    use super::*;
    use crate::{device_state::StoredAccountIdentity, store_ciphers::noop_cipher::NoopStoreCipher};

    fn logged_in(mode: StoredAccountMode) -> StoredDeviceState {
        StoredDeviceState::LoggedIn {
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

    fn storage(path: &Path) -> OnDiskDeviceStateStorage {
        OnDiskDeviceStateStorage::new(path, NoopStoreCipher)
    }

    #[tokio::test]
    async fn load_returns_logged_out_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("device_state.json"));

        assert_eq!(
            storage.load_device_state().await.unwrap(),
            StoredDeviceState::LoggedOut
        );
    }

    #[tokio::test]
    async fn store_and_load() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("nested/device_state.json"));

        let state = logged_in(StoredAccountMode::Api);
        storage.store_device_state(state.clone()).await.unwrap();

        assert_eq!(storage.load_device_state().await.unwrap(), state);
    }

    #[tokio::test]
    async fn store_overwrites_previous_state() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("device_state.json");
        let storage = storage(&path);

        storage
            .store_device_state(logged_in(StoredAccountMode::Api))
            .await
            .unwrap();
        storage
            .store_device_state(StoredDeviceState::LoggedOut)
            .await
            .unwrap();

        assert_eq!(
            storage.load_device_state().await.unwrap(),
            StoredDeviceState::LoggedOut
        );
        assert!(!path.with_added_extension("tmp").exists());
    }

    #[tokio::test]
    async fn remove_deletes_state() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("device_state.json");
        let storage = storage(&path);

        storage
            .store_device_state(logged_in(StoredAccountMode::Api))
            .await
            .unwrap();
        storage.remove_device_state().await.unwrap();

        assert!(!path.exists());
        assert_eq!(
            storage.load_device_state().await.unwrap(),
            StoredDeviceState::LoggedOut
        );
    }

    #[tokio::test]
    async fn remove_succeeds_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("device_state.json"));

        storage.remove_device_state().await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stored_file_is_private() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("device_state.json");
        let storage = storage(&path);

        storage
            .store_device_state(logged_in(StoredAccountMode::Api))
            .await
            .unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
