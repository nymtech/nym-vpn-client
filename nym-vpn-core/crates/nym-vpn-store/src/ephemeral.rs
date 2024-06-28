use tokio::sync::Mutex;

use crate::{DeviceKeys, KeyStore};

#[derive(Default)]
pub struct InMemEphemeralKeys {
    keys: Mutex<Option<DeviceKeys>>,
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
