// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::{KeyStore, DeviceKeys, OnDiskKeysError};
use std::path::Path;

pub async fn load_device_keys<P: AsRef<Path>>(path: P) -> Result<DeviceKeys, OnDiskKeysError> {
    let device_key_paths = nym_vpn_store::DeviceKeysPaths::new(path);
    let key_store = nym_vpn_store::OnDiskKeys::new(device_key_paths);

    key_store.load_keys().await
}
