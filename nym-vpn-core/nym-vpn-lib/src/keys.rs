// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::{DeviceKeyPair, DeviceKeys, KeyStore};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("failed to load device keys")]
    Load {
        path: PathBuf,
        error: nym_vpn_store::OnDiskKeysError,
    },

    #[error("failed to create device keys")]
    Create {
        path: PathBuf,
        error: nym_vpn_store::OnDiskKeysError,
    },

    #[error("failed to store device keys")]
    Store {
        path: PathBuf,
        error: nym_vpn_store::OnDiskKeysError,
    },
}

pub async fn load_device_keys<P: AsRef<Path> + Clone, T: DeviceKeyPair>(
    path: P,
) -> Result<DeviceKeys<T>, KeyStoreError> {
    let device_key_paths = nym_vpn_store::DeviceKeysPaths::new(path.clone());
    let key_store = nym_vpn_store::OnDiskKeys::new(device_key_paths);

    key_store
        .load_keys()
        .await
        .map_err(|error| KeyStoreError::Load {
            path: path.as_ref().to_path_buf(),
            error,
        })
}

pub async fn create_device_keys<P: AsRef<Path> + Clone>(path: P) -> Result<(), KeyStoreError> {
    let device_key_paths = nym_vpn_store::DeviceKeysPaths::new(path.clone());
    let key_store = nym_vpn_store::OnDiskKeys::new(device_key_paths);

    let mut rng = rand::rngs::OsRng;
    DeviceKeys::generate_new_ed25519(&mut rng)
        .persist_keys(&key_store)
        .await
        .map_err(|error| KeyStoreError::Create {
            path: path.as_ref().to_path_buf(),
            error,
        })
}

pub async fn store_device_keys<P: AsRef<Path> + Clone, T: DeviceKeyPair>(
    path: P,
    keys: &DeviceKeys<T>,
) -> Result<(), KeyStoreError> {
    let device_key_paths = nym_vpn_store::DeviceKeysPaths::new(path.clone());
    let key_store = nym_vpn_store::OnDiskKeys::new(device_key_paths);

    keys.persist_keys(&key_store)
        .await
        .map_err(|error| KeyStoreError::Store {
            path: path.as_ref().to_path_buf(),
            error,
        })
}
