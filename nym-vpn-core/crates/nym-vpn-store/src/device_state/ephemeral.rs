// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::convert::Infallible;

use tokio::sync::Mutex;

use super::{DeviceStateStorage, StoredDeviceState};

#[derive(Default)]
pub struct InMemoryDeviceStateStorage {
    state: Mutex<StoredDeviceState>,
}

#[async_trait::async_trait]
impl DeviceStateStorage for InMemoryDeviceStateStorage {
    type StorageError = Infallible;

    async fn load_device_state(&self) -> Result<StoredDeviceState, Self::StorageError> {
        Ok(self.state.lock().await.clone())
    }

    async fn store_device_state(&self, state: StoredDeviceState) -> Result<(), Self::StorageError> {
        *self.state.lock().await = state;
        Ok(())
    }

    async fn remove_device_state(&self) -> Result<(), Self::StorageError> {
        *self.state.lock().await = StoredDeviceState::LoggedOut;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use nym_vpn_lib_types::StoredAccountMode;

    use crate::device_state::StoredAccountIdentity;

    use super::*;

    fn logged_in() -> StoredDeviceState {
        StoredDeviceState::LoggedIn {
            account_identity: StoredAccountIdentity {
                id: "juno1cma4czt2jnydvrvz3lrc9jvcmhpjxtds95s3c6".to_owned(),
                mode: StoredAccountMode::Api,
                pub_key: "".to_owned(),
                signature_base64: "".to_owned(),
            },
            summary: None,
        }
    }

    #[tokio::test]
    async fn load_defaults_to_logged_out() {
        let storage = InMemoryDeviceStateStorage::default();

        assert_eq!(
            storage.load_device_state().await.unwrap(),
            StoredDeviceState::LoggedOut
        );
    }

    #[tokio::test]
    async fn store_and_load() {
        let storage = InMemoryDeviceStateStorage::default();
        storage.store_device_state(logged_in()).await.unwrap();

        assert_eq!(storage.load_device_state().await.unwrap(), logged_in());
    }

    #[tokio::test]
    async fn remove_resets_to_logged_out() {
        let storage = InMemoryDeviceStateStorage::default();
        storage.store_device_state(logged_in()).await.unwrap();
        storage.remove_device_state().await.unwrap();

        assert_eq!(
            storage.load_device_state().await.unwrap(),
            StoredDeviceState::LoggedOut
        );
    }
}
