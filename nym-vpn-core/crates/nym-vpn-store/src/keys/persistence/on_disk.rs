// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use nym_crypto::asymmetric::ed25519;
use nym_pemstore::{traits::PemStorableKeyPair, KeyPairPath};
use rand::SeedableRng as _;

use crate::keys::{DeviceKeys, KeyStore};

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
}

pub struct OnDiskKeys {
    paths: DeviceKeysPaths,
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
        self.private_device_key_file.exists()
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
    pub fn new(paths: DeviceKeysPaths) -> Self {
        OnDiskKeys { paths }
    }

    fn load_device_keypair(&self) -> Result<ed25519::KeyPair, OnDiskKeysError> {
        let device_paths = self.paths.device_key_pair_path();
        self.load_keypair(device_paths, "device")
    }

    fn load_keypair<T: PemStorableKeyPair>(
        &self,
        paths: KeyPairPath,
        name: impl Into<String>,
    ) -> Result<T, OnDiskKeysError> {
        nym_pemstore::load_keypair(&paths).map_err(|error| OnDiskKeysError::UnableToLoadKeys {
            paths,
            name: name.into(),
            error,
        })
    }

    fn store_keypair<T: PemStorableKeyPair>(
        &self,
        keypair: &T,
        paths: KeyPairPath,
        name: impl Into<String>,
    ) -> Result<(), OnDiskKeysError> {
        nym_pemstore::store_keypair(keypair, &paths).map_err(|error| {
            OnDiskKeysError::UnableToStoreKeys {
                paths,
                name: name.into(),
                error,
            }
        })
    }

    fn remove_keypair_files(&self, paths: &KeyPairPath) -> Result<(), std::io::Error> {
        std::fs::remove_file(&paths.private_key_path)?;
        std::fs::remove_file(&paths.public_key_path)
    }

    fn remove_keypair(&self, paths: KeyPairPath) -> Result<(), OnDiskKeysError> {
        self.remove_keypair_files(&paths)
            .map_err(|error| OnDiskKeysError::UnableToRemoveKeys { paths, error })
    }

    fn load_keys(&self) -> Result<DeviceKeys, OnDiskKeysError> {
        let device_keypair = self.load_device_keypair()?;
        Ok(DeviceKeys::from_keys(device_keypair))
    }

    fn store_keys(&self, keys: &DeviceKeys) -> Result<(), OnDiskKeysError> {
        let device_paths = self.paths.device_key_pair_path();
        self.store_keypair(keys.device_keypair().as_ref(), device_paths, "device")
    }

    // If there are no keys, generate them, otherwise do nothing
    fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), OnDiskKeysError> {
        if self.paths.exists() {
            return Ok(());
        }
        self.reset_keys(seed)
    }

    // Generate new keys and overwrite the existing ones if they exist
    fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), OnDiskKeysError> {
        let device_keys = if let Some(seed) = seed {
            let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed);
            DeviceKeys::generate_new(&mut rng)
        } else {
            let mut rng = rand::rngs::OsRng;
            DeviceKeys::generate_new(&mut rng)
        };
        self.store_keys(&device_keys)
    }

    fn remove_keys(&self) -> Result<(), OnDiskKeysError> {
        let device_paths = self.paths.device_key_pair_path();
        self.remove_keypair(device_paths)
    }
}

impl KeyStore for OnDiskKeys {
    type StorageError = OnDiskKeysError;

    async fn load_keys(&self) -> Result<DeviceKeys, Self::StorageError> {
        self.load_keys()
    }

    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError> {
        self.store_keys(keys)
    }

    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError> {
        self.init_keys(seed)
    }

    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError> {
        self.reset_keys(seed)
    }

    async fn remove_keys(&self) -> Result<(), Self::StorageError> {
        self.reset_keys(None)
            .inspect_err(|_| {
                tracing::warn!("Failed to reset keys before removal.");
            })
            .ok();
        self.remove_keys()
    }
}
