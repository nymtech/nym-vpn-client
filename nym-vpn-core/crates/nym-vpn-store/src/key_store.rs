// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use crate::DeviceKeys;

pub trait KeyStore {
    type StorageError: Error;

    #[allow(async_fn_in_trait)]
    async fn load_keys(&self) -> Result<DeviceKeys, Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError>;
}

