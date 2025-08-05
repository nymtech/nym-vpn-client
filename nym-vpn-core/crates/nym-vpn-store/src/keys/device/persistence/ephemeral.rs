// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use rand::SeedableRng as _;
use tokio::sync::Mutex;

use crate::keys::device::{DeviceKeyStore, DeviceKeys};

#[derive(Default)]
pub struct InMemEphemeralKeys {
    keys: Mutex<Option<DeviceKeys>>,
}

#[async_trait::async_trait]
impl DeviceKeyStore for InMemEphemeralKeys {
    type StorageError = std::convert::Infallible;

    async fn load_keys(&self) -> Result<Option<DeviceKeys>, Self::StorageError> {
        Ok(self.keys.lock().await.clone())
    }

    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError> {
        *self.keys.lock().await = Some(keys.clone());
        Ok(())
    }

    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError> {
        if self.load_keys().await?.is_some() {
            return Ok(());
        }
        self.reset_keys(seed).await
    }

    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError> {
        let device_keys = if let Some(seed) = seed {
            let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed);
            DeviceKeys::generate_new(&mut rng)
        } else {
            let mut rng = rand::rngs::OsRng;
            DeviceKeys::generate_new(&mut rng)
        };
        self.store_keys(&device_keys).await
    }

    async fn remove_keys(&self) -> Result<(), Self::StorageError> {
        *self.keys.lock().await = None;
        Ok(())
    }
}
