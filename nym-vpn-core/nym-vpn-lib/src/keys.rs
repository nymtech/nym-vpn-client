// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use nym_crypto::{asymmetric::identity, ctr::cipher::zeroize::ZeroizeOnDrop};
use nym_pemstore::{traits::PemStorableKeyPair, KeyPairPath};
use rand::{CryptoRng, RngCore};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct DeviceKeys {
    device_keypair: Arc<identity::KeyPair>,
}

impl DeviceKeys {
    pub fn generate_new<R>(rng: &mut R) -> Self
    where
        R: RngCore + CryptoRng,
    {
        DeviceKeys {
            device_keypair: Arc::new(identity::KeyPair::new(rng)),
        }
    }

    pub fn from_keys(device_keypair: identity::KeyPair) -> Self {
        DeviceKeys {
            device_keypair: Arc::new(device_keypair),
        }
    }

    pub async fn load_keys<S: KeyStore>(store: &S) -> Result<Self, S::StorageError> {
        store.load_keys().await
    }

    pub async fn persist_keys<S: KeyStore>(&self, store: &S) -> Result<(), S::StorageError> {
        store.store_keys(self).await
    }
}

fn _assert_keys_zeroize_on_drop() {
    fn _assert_zeroize_on_drop<T: ZeroizeOnDrop>() {}

    _assert_zeroize_on_drop::<identity::KeyPair>();
}

pub trait KeyStore {
    type StorageError: Error;

    async fn load_keys(&self) -> Result<DeviceKeys, Self::StorageError>;
    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError>;
}

#[derive(Default)]
pub struct InMemEphemeralKeys {
    keys: Mutex<Option<DeviceKeys>>,
}

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
    private_device_key_file: PathBuf,
    public_device_key_file: PathBuf,
}

impl DeviceKeysPaths {
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

#[derive(Debug, thiserror::Error)]
pub enum EphemeralKeysError {
    #[error("unable to load ephemeral keys")]
    UnableToLoadKeys,
}

impl KeyStore for InMemEphemeralKeys {
    type StorageError = EphemeralKeysError;

    async fn load_keys(&self) -> Result<DeviceKeys, Self::StorageError> {
        self.keys
            .lock()
            .await
            .as_ref()
            .cloned()
            .ok_or(EphemeralKeysError::UnableToLoadKeys)
    }

    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError> {
        *self.keys.lock().await = Some(keys.clone());
        Ok(())
    }
}

pub async fn generate_new_device_keys<K, R>(rng: &mut R, key_store: &K) -> Result<(), String>
where
    R: RngCore + CryptoRng,
    K: KeyStore,
{
    DeviceKeys::generate_new(rng)
        .persist_keys(key_store)
        .await
        .map_err(|_| "Failed to persist device keys".to_string())
}

pub async fn load_device_keys<K>(key_store: &K) -> Result<DeviceKeys, String>
where
    K: KeyStore,
{
    DeviceKeys::load_keys(key_store)
        .await
        .map_err(|_| "Failed to load device keys".to_string())
}

pub async fn store_device_keys<K>(keys: &DeviceKeys, key_store: &K) -> Result<(), String>
where
    K: KeyStore,
{
    keys.persist_keys(key_store)
        .await
        .map_err(|_| "Failed to store device keys".to_string())
}
