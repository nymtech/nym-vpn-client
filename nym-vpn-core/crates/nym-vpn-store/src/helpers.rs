use std::error::Error;

use rand::{CryptoRng, RngCore};

use crate::{key_store::KeyStore, DeviceKeys};

// Helper functions for error wrapping

#[derive(Debug, thiserror::Error)]
pub enum VpnStoreError {
    #[error("failed to generate keys")]
    GenerateKeys {
        source: Box<dyn Error + Send + Sync + 'static>,
    },

    #[error("failed to load keys")]
    LoadKeys {
        source: Box<dyn Error + Send + Sync + 'static>,
    },

    #[error("failed to store keys")]
    StoreKeys {
        source: Box<dyn Error + Send + Sync + 'static>,
    },
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
        .map_err(|err| VpnStoreError::GenerateKeys {
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
        .map_err(|err| VpnStoreError::LoadKeys {
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
        .map_err(|err| VpnStoreError::StoreKeys {
            source: Box::new(err),
        })
}
