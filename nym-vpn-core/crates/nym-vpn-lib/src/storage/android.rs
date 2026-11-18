// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::{
    database_key::{DatabaseKey, DatabaseKeyStorageError, on_disk::OnDiskDatabaseKeyStorage},
    keys::device::DeviceKeysPaths,
    secure_store_migration::SecureStoreMigrationManager,
    store_ciphers::secure_enclave_cipher::{SecureEnclaveCipher, SecureEnclaveCipherOptions},
};

use super::{LEGACY_MNEMONIC_FILE_NAME, StorageConfig};

pub type PlatformMnemonicStorage = nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorage;
pub type PlatformAccountStorage = nym_vpn_store::account::on_disk::OnDiskAccountStorage;
pub type PlatformDeviceKeys = nym_vpn_store::keys::device::OnDiskKeys;

const MIGRATION_UNLOCK_REASON: &str =
    "Unlock your account credentials to perform migration from plain to encrypted account storage";

const MNEMONIC_FILE_NAME: &str = "mnemonic.bin";
const ACCOUNT_INFO_FILE_NAME: &str = "account.bin";
const DATABASE_KEY_FILE_NAME: &str = "encryption.key";

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

/// Loads the database key, encrypted with a key held by the Android keystore.
pub async fn database_key(config: &StorageConfig) -> Result<DatabaseKey, DatabaseKeyStorageError> {
    OnDiskDatabaseKeyStorage::new(
        config.base_data_directory.join(DATABASE_KEY_FILE_NAME),
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

pub fn device_keys(config: &StorageConfig, database_key: &DatabaseKey) -> PlatformDeviceKeys {
    PlatformDeviceKeys::new(
        DeviceKeysPaths::new(&config.base_data_directory),
        database_key,
    )
}

/// The keystore is unlocked before migrating, since the mnemonic key requires user presence.
pub fn migration_manager<'a>(
    config: &StorageConfig,
    mnemonic_storage: &'a PlatformMnemonicStorage,
    account_storage: &'a PlatformAccountStorage,
) -> SecureStoreMigrationManager<'a, PlatformMnemonicStorage, PlatformAccountStorage> {
    SecureStoreMigrationManager::new(
        config.base_data_directory.join(LEGACY_MNEMONIC_FILE_NAME),
        mnemonic_storage,
        account_storage,
        config.secure_enclave.clone(),
        MIGRATION_UNLOCK_REASON,
    )
}
