// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error, sync::Arc};

use nym_crypto::{asymmetric::identity, ctr::cipher::zeroize::ZeroizeOnDrop};
use rand::{CryptoRng, RngCore};

pub use ephemeral::InMemEphemeralKeys;
pub use on_disk::{DeviceKeysPaths, OnDiskKeys};

mod ephemeral;
mod on_disk;

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

    #[allow(async_fn_in_trait)]
    async fn load_keys(&self) -> Result<DeviceKeys, Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError>;
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
