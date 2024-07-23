// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use crate::{device_keys::DeviceKeyPair, DeviceKeys};

pub trait KeyStore<T: DeviceKeyPair> {
    type StorageError: Error;

    #[allow(async_fn_in_trait)]
    async fn load_keys(&self) -> Result<DeviceKeys<T>, Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn store_keys(&self, keys: &DeviceKeys<T>) -> Result<(), Self::StorageError>;
}
