// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use nym_crypto::asymmetric::identity;
use nym_pemstore::{traits::PemStorableKeyPair, KeyPairPath};

use crate::{DeviceKeys, KeyStore};

#[derive(Debug, thiserror::Error)]
pub enum OnDiskKeysError {
    #[error("unable to load keys")]
    UnableToLoadKeys,

    #[error("unable to store keys")]
    UnableToStoreKeys,
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
            private_device_key_file: base_dir.join("private_device.pem"),
            public_device_key_file: base_dir.join("public_device.pem"),
        }
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

    fn load_device_keypair(&self) -> Result<identity::KeyPair, OnDiskKeysError> {
        let device_paths = self.paths.device_key_pair_path();
        self.load_keypair(device_paths, "device")
    }

    fn load_keypair<T: PemStorableKeyPair>(
        &self,
        paths: KeyPairPath,
        _name: impl Into<String>,
    ) -> Result<T, OnDiskKeysError> {
        nym_pemstore::load_keypair(&paths).map_err(|_| OnDiskKeysError::UnableToLoadKeys)
    }

    fn store_keypair<T: PemStorableKeyPair>(
        &self,
        keypair: &T,
        paths: KeyPairPath,
        _name: impl Into<String>,
    ) -> Result<(), OnDiskKeysError> {
        nym_pemstore::store_keypair(keypair, &paths).map_err(|_| OnDiskKeysError::UnableToStoreKeys)
    }

    fn load_keys(&self) -> Result<DeviceKeys, OnDiskKeysError> {
        let device_keypair = self.load_device_keypair()?;
        Ok(DeviceKeys::from_keys(device_keypair))
    }

    fn store_keys(&self, keys: &DeviceKeys) -> Result<(), OnDiskKeysError> {
        let device_paths = self.paths.device_key_pair_path();
        self.store_keypair(keys.device_keypair.as_ref(), device_paths, "device")
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
}
