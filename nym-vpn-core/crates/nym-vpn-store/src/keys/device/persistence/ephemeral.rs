// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use rand::SeedableRng as _;
use tokio::sync::Mutex;

use crate::keys::device::{DeviceKeyStore, DeviceKeys};

#[derive(Default)]
pub struct InMemEphemeralKeys {
    keys: Mutex<Option<DeviceKeys>>,
}

#[derive(Debug, thiserror::Error)]
pub enum EphemeralKeysError {
    #[error("unable to load ephemeral keys")]
    UnableToLoadKeys,
}

#[async_trait::async_trait]
impl DeviceKeyStore for InMemEphemeralKeys {
    type StorageError = EphemeralKeysError;

    async fn load_keys(&self) -> Result<DeviceKeys, EphemeralKeysError> {
        self.keys
            .lock()
            .await
            .as_ref()
            .cloned()
            .ok_or(EphemeralKeysError::UnableToLoadKeys)
    }

    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), EphemeralKeysError> {
        *self.keys.lock().await = Some(keys.clone());
        Ok(())
    }

    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), EphemeralKeysError> {
        if self.load_keys().await.is_ok() {
            return Ok(());
        }
        self.reset_keys(seed).await
    }

    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), EphemeralKeysError> {
        let device_keys = if let Some(seed) = seed {
            let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed);
            DeviceKeys::generate_new(&mut rng)
        } else {
            let mut rng = rand::rngs::OsRng;
            DeviceKeys::generate_new(&mut rng)
        };
        self.store_keys(&device_keys).await
    }

    async fn remove_keys(&self) -> Result<(), EphemeralKeysError> {
        *self.keys.lock().await = None;
        Ok(())
    }
}
