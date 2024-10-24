// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use super::DeviceKeys;

pub trait KeyStore {
    type StorageError: Error + Send + Sync + 'static;

    #[allow(async_fn_in_trait)]
    async fn load_keys(&self) -> Result<DeviceKeys, Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError>;
}
