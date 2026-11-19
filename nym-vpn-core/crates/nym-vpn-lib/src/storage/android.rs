// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::{
    keys::device::DeviceKeysPaths,
    master_key::{KeySecret, MasterKey, MasterKeyStorageError, on_disk::OnDiskMasterKeyStorage},
    store_ciphers::secure_enclave_cipher::{SecureEnclaveCipher, SecureEnclaveCipherOptions},
};

use super::StorageConfig;

pub type PlatformMnemonicStorage = nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorage;
pub type PlatformAccountStorage = nym_vpn_store::account::on_disk::OnDiskAccountStorage;
pub type PlatformDeviceKeys = nym_vpn_store::keys::device::OnDiskKeys;

pub const MIGRATION_UNLOCK_REASON: &str =
    "Unlock your account credentials to perform migration from plain to encrypted account storage";

const MNEMONIC_FILE_NAME: &str = "mnemonic.bin";
const ACCOUNT_INFO_FILE_NAME: &str = "account.bin";
const MASTER_KEY_FILE_NAME: &str = "master.key";

/// Key used for mnemonic encryption (checked user presence)
const MNEMONIC_SE_KEY: &str = "mnemonic-key";

/// Key used for encryption of all other files (unattended)
const DEVICE_SE_KEY: &str = "device-key";

/// Cipher for the mnemonic, requiring user presence.
fn mnemonic_cipher(config: &StorageConfig) -> SecureEnclaveCipher {
    SecureEnclaveCipher::new(
        config.secure_enclave.clone(),
        SecureEnclaveCipherOptions {
            key_alias: MNEMONIC_SE_KEY.to_owned(),
            user_presence: true,
        },
    )
}

/// Cipher for all other files, usable unattended.
fn device_cipher(config: &StorageConfig) -> SecureEnclaveCipher {
    SecureEnclaveCipher::new(
        config.secure_enclave.clone(),
        SecureEnclaveCipherOptions {
            key_alias: DEVICE_SE_KEY.to_owned(),
            user_presence: false,
        },
    )
}

/// Loads the master key, encrypted with a key held by the Android keystore.
pub async fn master_key(config: &StorageConfig) -> Result<MasterKey, MasterKeyStorageError> {
    OnDiskMasterKeyStorage::new(
        config.base_data_directory.join(MASTER_KEY_FILE_NAME),
        device_cipher(config),
    )
    .load_or_create()
    .await
}

pub fn mnemonic_storage(config: &StorageConfig) -> PlatformMnemonicStorage {
    PlatformMnemonicStorage::new(
        config.base_data_directory.join(MNEMONIC_FILE_NAME),
        mnemonic_cipher(config),
    )
}

pub fn account_storage(config: &StorageConfig) -> PlatformAccountStorage {
    PlatformAccountStorage::new(
        config.base_data_directory.join(ACCOUNT_INFO_FILE_NAME),
        device_cipher(config),
    )
}

pub fn device_keys(config: &StorageConfig, key_secret: KeySecret) -> PlatformDeviceKeys {
    PlatformDeviceKeys::new(
        DeviceKeysPaths::new(&config.base_data_directory),
        key_secret,
    )
}
