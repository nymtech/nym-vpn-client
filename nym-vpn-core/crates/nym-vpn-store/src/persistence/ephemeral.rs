// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::Mutex;

use crate::{device_keys::DeviceKeyPair, DeviceKeys, KeyStore};

#[derive(Default)]
pub struct InMemEphemeralKeys<T: DeviceKeyPair> {
    keys: Mutex<Option<DeviceKeys<T>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum EphemeralKeysError {
    #[error("unable to load ephemeral keys")]
    UnableToLoadKeys,
}

impl<T: Clone + DeviceKeyPair> KeyStore<T> for InMemEphemeralKeys<T> {
    type StorageError = EphemeralKeysError;

    async fn load_keys(&self) -> Result<DeviceKeys<T>, Self::StorageError> {
        self.keys
            .lock()
            .await
            .as_ref()
            .cloned()
            .ok_or(EphemeralKeysError::UnableToLoadKeys)
    }

    async fn store_keys(&self, keys: &DeviceKeys<T>) -> Result<(), Self::StorageError> {
        *self.keys.lock().await = Some(keys.clone());
        Ok(())
    }
}
