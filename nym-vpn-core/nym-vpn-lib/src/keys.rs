// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::{DeviceKeys, VpnStoreError};
use std::path::Path;

pub async fn load_device_keys<P: AsRef<Path>>(path: P) -> Result<DeviceKeys, VpnStoreError> {
    let device_key_paths = nym_vpn_store::DeviceKeysPaths::new(path);
    let key_store = nym_vpn_store::OnDiskKeys::new(device_key_paths);

    nym_vpn_store::load_device_keys(&key_store).await
}

pub async fn create_device_keys<P: AsRef<Path>>(path: P) -> Result<(), VpnStoreError> {
    let device_key_paths = nym_vpn_store::DeviceKeysPaths::new(path);
    let key_store = nym_vpn_store::OnDiskKeys::new(device_key_paths);

    let mut rng = rand::rngs::OsRng;
    nym_vpn_store::generate_new_device_keys(&mut rng, &key_store).await
}

pub async fn store_device_keys<P: AsRef<Path>>(
    path: P,
    keys: &DeviceKeys,
) -> Result<(), VpnStoreError> {
    let device_key_paths = nym_vpn_store::DeviceKeysPaths::new(path);
    let key_store = nym_vpn_store::OnDiskKeys::new(device_key_paths);

    nym_vpn_store::store_device_keys(keys, &key_store).await
}
