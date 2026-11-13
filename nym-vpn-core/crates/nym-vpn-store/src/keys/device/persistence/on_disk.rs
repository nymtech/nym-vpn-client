// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use nym_crypto::asymmetric::ed25519;
use nym_pemstore::{KeyPairPath, Passphrase, traits::PemStorableKeyPair};
use rand::SeedableRng;

use crate::{
    database_key::{DatabaseKeyStorageError, OnDiskDatabaseKeyStorage},
    keys::device::{DeviceKeyStore, DeviceKeys},
};

pub const DEFAULT_PRIVATE_DEVICE_KEY_FILENAME: &str = "private_device.pem";
pub const DEFAULT_PUBLIC_DEVICE_KEY_FILENAME: &str = "public_device.pem";

#[derive(Debug, thiserror::Error)]
pub enum OnDiskKeysError {
    #[error("unable to load keys")]
    UnableToLoadKeys {
        paths: KeyPairPath,
        name: String,
        error: std::io::Error,
    },

    #[error("unable to store keys")]
    UnableToStoreKeys {
        paths: KeyPairPath,
        name: String,
        error: std::io::Error,
    },

    #[error("unable to remove keys")]
    UnableToRemoveKeys {
        paths: KeyPairPath,
        error: std::io::Error,
    },

    #[error("unable to check whether keys are encrypted")]
    UnableToCheckEncryption {
        paths: KeyPairPath,
        error: std::io::Error,
    },

    #[error("failed to load encryption key")]
    LoadEncryptionKey(#[source] DatabaseKeyStorageError),
}

/// Stores device keys in PEM files, with the private key encrypted with the database key.
pub struct OnDiskKeys {
    paths: DeviceKeysPaths,
    database_key_storage: Arc<OnDiskDatabaseKeyStorage>,
}

pub struct DeviceKeysPaths {
    pub private_device_key_file: PathBuf,
    pub public_device_key_file: PathBuf,
}

impl DeviceKeysPaths {
    pub fn new<P: AsRef<Path>>(base_data_directory: P) -> Self {
        let base_dir = base_data_directory.as_ref();
        DeviceKeysPaths {
            private_device_key_file: base_dir.join(DEFAULT_PRIVATE_DEVICE_KEY_FILENAME),
            public_device_key_file: base_dir.join(DEFAULT_PUBLIC_DEVICE_KEY_FILENAME),
        }
    }

    pub fn exists(&self) -> bool {
        self.private_device_key_file.exists() && self.public_device_key_file.exists()
    }

    pub fn device_key_pair_path(&self) -> nym_pemstore::KeyPairPath {
        nym_pemstore::KeyPairPath::new(
            self.private_device_key().to_path_buf(),
            self.public_device_key().to_path_buf(),
        )
    }

    pub fn private_device_key(&self) -> &Path {
        &self.private_device_key_file
    }

    pub fn public_device_key(&self) -> &Path {
        &self.public_device_key_file
    }
}

impl OnDiskKeys {
    pub fn new(
        paths: DeviceKeysPaths,
        database_key_storage: Arc<OnDiskDatabaseKeyStorage>,
    ) -> Self {
        OnDiskKeys {
            paths,
            database_key_storage,
        }
    }

    async fn passphrase(&self) -> Result<Passphrase, OnDiskKeysError> {
        let database_key = self
            .database_key_storage
            .load_or_create()
            .await
            .map_err(OnDiskKeysError::LoadEncryptionKey)?;
        Ok(Passphrase::new(hex::encode(database_key.as_bytes())))
    }

    async fn load_device_keypair(&self) -> Result<Option<ed25519::KeyPair>, OnDiskKeysError> {
        let device_paths = self.paths.device_key_pair_path();
        let passphrase = self.passphrase().await?;
        match self.load_keypair(device_paths.clone(), "device", &passphrase) {
            Ok(keypair) => {
                self.encrypt_plaintext_keypair(&keypair, device_paths, "device", &passphrase)?;
                Ok(Some(keypair))
            }
            Err(OnDiskKeysError::UnableToLoadKeys { error, .. })
                if error.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Re-stores a keypair whose private key was stored before encryption was introduced.
    fn encrypt_plaintext_keypair<T: PemStorableKeyPair>(
        &self,
        keypair: &T,
        paths: KeyPairPath,
        name: &str,
        passphrase: &Passphrase,
    ) -> Result<(), OnDiskKeysError> {
        let is_encrypted =
            nym_pemstore::is_encrypted(&paths.private_key_path).map_err(|error| {
                OnDiskKeysError::UnableToCheckEncryption {
                    paths: paths.clone(),
                    error,
                }
            })?;
        if is_encrypted {
            Ok(())
        } else {
            tracing::info!("Encrypting {name} keys");
            self.store_keypair(keypair, paths, name, passphrase)
        }
    }

    fn load_keypair<T: PemStorableKeyPair>(
        &self,
        paths: KeyPairPath,
        name: impl Into<String>,
        passphrase: &Passphrase,
    ) -> Result<T, OnDiskKeysError> {
        nym_pemstore::load_keypair_with(&paths, Some(passphrase)).map_err(|error| {
            OnDiskKeysError::UnableToLoadKeys {
                paths,
                name: name.into(),
                error,
            }
        })
    }

    fn store_keypair<T: PemStorableKeyPair>(
        &self,
        keypair: &T,
        paths: KeyPairPath,
        name: impl Into<String>,
        passphrase: &Passphrase,
    ) -> Result<(), OnDiskKeysError> {
        nym_pemstore::store_keypair_with(keypair, &paths, Some(passphrase)).map_err(|error| {
            OnDiskKeysError::UnableToStoreKeys {
                paths,
                name: name.into(),
                error,
            }
        })
    }

    async fn remove_keypair_files(&self, paths: &KeyPairPath) -> std::io::Result<()> {
        for f in [&paths.private_key_path, &paths.public_key_path] {
            tokio::fs::remove_file(f).await?;
        }
        Ok(())
    }

    async fn remove_keypair(&self, paths: KeyPairPath) -> Result<(), OnDiskKeysError> {
        self.remove_keypair_files(&paths)
            .await
            .map_err(|error| OnDiskKeysError::UnableToRemoveKeys { paths, error })
    }

    async fn load_keys(&self) -> Result<Option<DeviceKeys>, OnDiskKeysError> {
        let device_keypair = self.load_device_keypair().await?;
        Ok(device_keypair.map(DeviceKeys::from_keys))
    }

    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), OnDiskKeysError> {
        let device_paths = self.paths.device_key_pair_path();
        let passphrase = self.passphrase().await?;
        self.store_keypair(
            keys.device_keypair().as_ref(),
            device_paths,
            "device",
            &passphrase,
        )
    }

    // If there are no keys, generate them, otherwise do nothing
    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), OnDiskKeysError> {
        if self.paths.exists() {
            return Ok(());
        }
        self.reset_keys(seed).await
    }

    // Generate new keys and overwrite the existing ones if they exist
    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), OnDiskKeysError> {
        let device_keys = if let Some(seed) = seed {
            let mut rng = rand::rngs::ChaCha20Rng::from_seed(seed);
            DeviceKeys::generate_new(&mut rng)
        } else {
            DeviceKeys::generate_new(&mut rand::rng())
        };
        self.store_keys(&device_keys).await
    }

    async fn remove_keys(&self) -> Result<(), OnDiskKeysError> {
        let device_paths = self.paths.device_key_pair_path();
        self.remove_keypair(device_paths).await
    }
}

#[async_trait::async_trait]
impl DeviceKeyStore for OnDiskKeys {
    type StorageError = OnDiskKeysError;

    async fn load_keys(&self) -> Result<Option<DeviceKeys>, OnDiskKeysError> {
        self.load_keys().await
    }

    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), OnDiskKeysError> {
        self.store_keys(keys).await
    }

    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), OnDiskKeysError> {
        self.init_keys(seed).await
    }

    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), OnDiskKeysError> {
        self.reset_keys(seed).await
    }

    async fn remove_keys(&self) -> Result<(), OnDiskKeysError> {
        self.reset_keys(None)
            .await
            .inspect_err(|_| {
                tracing::warn!("Failed to reset keys before removal.");
            })
            .ok();
        self.remove_keys().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store_ciphers::noop_cipher::NoopStoreCipher;

    fn on_disk_keys(dir: &Path) -> OnDiskKeys {
        let database_key_storage = Arc::new(OnDiskDatabaseKeyStorage::new(
            dir.join("encryption.key"),
            NoopStoreCipher,
        ));
        OnDiskKeys::new(DeviceKeysPaths::new(dir), database_key_storage)
    }

    fn public_key(keys: &DeviceKeys) -> String {
        keys.device_keypair().public_key().to_base58_string()
    }

    #[tokio::test]
    async fn stores_private_key_encrypted() {
        let tempdir = tempfile::tempdir().unwrap();
        let store = on_disk_keys(tempdir.path());
        let keys = DeviceKeys::generate_new(&mut rand::rng());

        store.store_keys(&keys).await.unwrap();

        let paths = DeviceKeysPaths::new(tempdir.path());
        assert!(nym_pemstore::is_encrypted(paths.private_device_key()).unwrap());
        assert!(!nym_pemstore::is_encrypted(paths.public_device_key()).unwrap());

        let loaded = on_disk_keys(tempdir.path())
            .load_keys()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(public_key(&loaded), public_key(&keys));
    }

    #[tokio::test]
    async fn encrypts_plaintext_keys_on_load() {
        let tempdir = tempfile::tempdir().unwrap();
        let paths = DeviceKeysPaths::new(tempdir.path());
        let keys = DeviceKeys::generate_new(&mut rand::rng());
        nym_pemstore::store_keypair(
            keys.device_keypair().as_ref(),
            &paths.device_key_pair_path(),
        )
        .unwrap();

        let loaded = on_disk_keys(tempdir.path())
            .load_keys()
            .await
            .unwrap()
            .unwrap();

        assert_eq!(public_key(&loaded), public_key(&keys));
        assert!(nym_pemstore::is_encrypted(paths.private_device_key()).unwrap());
    }

    #[tokio::test]
    async fn missing_keys_load_as_none() {
        let tempdir = tempfile::tempdir().unwrap();

        let loaded = on_disk_keys(tempdir.path()).load_keys().await.unwrap();

        assert!(loaded.is_none());
    }
}
