// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// use nym_vpn_store::{
//     keys::{
//         persistence::on_disk::{DeviceKeysPaths, OnDiskKeysError},
//         DeviceKeys, KeyStore,
//     },
//     mnemonic::{on_disk::OnDiskMnemonicStorageError, Mnemonic, MnemonicStorage},
// };
//
// use std::path::{Path, PathBuf};
//
// pub async fn load_device_keys<P: AsRef<Path> + Clone>(
//     path: P,
// ) -> Result<DeviceKeys, KeyStoreError> {
//     let vpn_storage = VpnClientOnDiskStorage::new(path.clone());
//
//     vpn_storage
//         .load_keys()
//         .await
//         .map_err(|error| KeyStoreError::Load {
//             path: path.as_ref().to_path_buf(),
//             error,
//         })
// }

// pub async fn create_device_keys<P: AsRef<Path> + Clone>(path: P) -> Result<(), KeyStoreError> {
//     let device_key_paths = DeviceKeysPaths::new(path.clone());
//     let key_store = nym_vpn_store::keys::persistence::on_disk::OnDiskKeys::new(device_key_paths);
//
//     let mut rng = rand::rngs::OsRng;
//     DeviceKeys::generate_new(&mut rng)
//         .persist_keys(&key_store)
//         .await
//         .map_err(|error| KeyStoreError::Create {
//             path: path.as_ref().to_path_buf(),
//             error,
//         })
// }
//
// pub async fn store_device_keys<P: AsRef<Path> + Clone>(
//     path: P,
//     keys: &DeviceKeys,
// ) -> Result<(), KeyStoreError> {
//     let device_key_paths = DeviceKeysPaths::new(path.clone());
//     let key_store = nym_vpn_store::keys::persistence::on_disk::OnDiskKeys::new(device_key_paths);
//
//     keys.persist_keys(&key_store)
//         .await
//         .map_err(|error| KeyStoreError::Store {
//             path: path.as_ref().to_path_buf(),
//             error,
//         })
// }
