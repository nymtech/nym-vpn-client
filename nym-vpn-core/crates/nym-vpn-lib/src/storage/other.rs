// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::{
    keys::device::DeviceKeysPaths,
    master_key::{KeySecret, MasterKey, MasterKeyStorageError, on_disk::OnDiskMasterKeyStorage},
    store_ciphers::noop_cipher::NoopStoreCipher,
};

use super::StorageConfig;

pub type PlatformMnemonicStorage = nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorage;
pub type PlatformAccountStorage = nym_vpn_store::account::on_disk::OnDiskAccountStorage;
pub type PlatformDeviceKeys = nym_vpn_store::keys::device::OnDiskKeys;

const MNEMONIC_FILE_NAME: &str = "mnemonic.bin";
const ACCOUNT_INFO_FILE_NAME: &str = "account.bin";
const MASTER_KEY_FILE_NAME: &str = "master.key";

/// Loads the master key from a file without encryption, relying on file permissions.
pub async fn master_key(config: &StorageConfig) -> Result<MasterKey, MasterKeyStorageError> {
    OnDiskMasterKeyStorage::new(
        config.base_data_directory.join(MASTER_KEY_FILE_NAME),
        NoopStoreCipher,
    )
    .load_or_create()
    .await
}

pub fn mnemonic_storage(config: &StorageConfig) -> PlatformMnemonicStorage {
    PlatformMnemonicStorage::new(
        config.base_data_directory.join(MNEMONIC_FILE_NAME),
        NoopStoreCipher,
    )
}

pub fn account_storage(config: &StorageConfig) -> PlatformAccountStorage {
    PlatformAccountStorage::new(
        config.base_data_directory.join(ACCOUNT_INFO_FILE_NAME),
        NoopStoreCipher,
    )
}

pub fn device_keys(config: &StorageConfig, key_secret: KeySecret) -> PlatformDeviceKeys {
    PlatformDeviceKeys::new(
        DeviceKeysPaths::new(&config.base_data_directory),
        key_secret,
    )
}
