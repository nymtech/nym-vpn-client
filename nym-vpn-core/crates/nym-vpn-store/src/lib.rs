// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error, sync::Arc};

use nym_crypto::{asymmetric::identity, ctr::cipher::zeroize::ZeroizeOnDrop};
use rand::{CryptoRng, RngCore};

pub use ephemeral::InMemEphemeralKeys;
pub use on_disk::{DeviceKeysPaths, OnDiskKeys, OnDiskKeysError};

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

// Helper functions for error wrapping

#[derive(Debug, thiserror::Error)]
pub enum VpnStoreError {
    #[error("failed to generate keys")]
    FailedToGenerateKeys {
        source: Box<dyn Error + Send + Sync + 'static>,
    },

    #[error("failed to load keys")]
    FailedToLoadKeys {
        source: Box<dyn Error + Send + Sync + 'static>,
    },

    #[error("failed to store keys")]
    FailedToStoreKeys {
        source: Box<dyn Error + Send + Sync + 'static>,
    }
}

pub async fn generate_new_device_keys<K, R>(rng: &mut R, key_store: &K) -> Result<(), VpnStoreError>
where
    R: RngCore + CryptoRng,
    K: KeyStore,
    K::StorageError: Send + Sync + 'static,
{
    DeviceKeys::generate_new(rng)
        .persist_keys(key_store)
        .await
        .map_err(|err| VpnStoreError::FailedToGenerateKeys {
            source: Box::new(err),
        })
}

pub async fn load_device_keys<K>(key_store: &K) -> Result<DeviceKeys, VpnStoreError>
where
    K: KeyStore,
    K::StorageError: Send + Sync + 'static,
{
    DeviceKeys::load_keys(key_store)
        .await
        .map_err(|err| VpnStoreError::FailedToLoadKeys {
            source: Box::new(err),
        })
}

pub async fn store_device_keys<K>(keys: &DeviceKeys, key_store: &K) -> Result<(), VpnStoreError>
where
    K: KeyStore,
    K::StorageError: Send + Sync + 'static,
{
    keys.persist_keys(key_store)
        .await
        .map_err(|err| VpnStoreError::FailedToStoreKeys {
            source: Box::new(err),
        })
}
