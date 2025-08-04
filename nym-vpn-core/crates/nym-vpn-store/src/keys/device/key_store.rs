// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use super::keys::DeviceKeys;

#[async_trait::async_trait]
pub trait DeviceKeyStore {
    type StorageError: Error + Send + Sync + 'static;

    async fn load_keys(&self) -> Result<Option<DeviceKeys>, Self::StorageError>; // None means no error, but no keys stored
    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError>;
    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError>;
    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError>;
    async fn remove_keys(&self) -> Result<(), Self::StorageError>;
}
