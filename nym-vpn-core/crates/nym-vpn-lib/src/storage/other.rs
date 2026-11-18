// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::{
    database_key::{DatabaseKey, DatabaseKeyStorageError, on_disk::OnDiskDatabaseKeyStorage},
    keys::device::DeviceKeysPaths,
    secure_store_migration::SecureStoreMigrationManager,
    store_ciphers::noop_cipher::NoopStoreCipher,
};

use super::{LEGACY_MNEMONIC_FILE_NAME, StorageConfig};

pub type PlatformMnemonicStorage = nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorage;
pub type PlatformAccountStorage = nym_vpn_store::account::on_disk::OnDiskAccountStorage;
pub type PlatformDeviceKeys = nym_vpn_store::keys::device::OnDiskKeys;

const MNEMONIC_FILE_NAME: &str = "mnemonic.bin";
const ACCOUNT_INFO_FILE_NAME: &str = "account.bin";
const DATABASE_KEY_FILE_NAME: &str = "encryption.key";

/// Loads the database key from a file without encryption, relying on file permissions.
pub async fn database_key(config: &StorageConfig) -> Result<DatabaseKey, DatabaseKeyStorageError> {
    OnDiskDatabaseKeyStorage::new(
        config.base_data_directory.join(DATABASE_KEY_FILE_NAME),
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

pub fn device_keys(config: &StorageConfig, database_key: &DatabaseKey) -> PlatformDeviceKeys {
    PlatformDeviceKeys::new(
        DeviceKeysPaths::new(&config.base_data_directory),
        database_key,
    )
}

pub fn migration_manager<'a>(
    config: &StorageConfig,
    mnemonic_storage: &'a PlatformMnemonicStorage,
    account_storage: &'a PlatformAccountStorage,
) -> SecureStoreMigrationManager<'a, PlatformMnemonicStorage, PlatformAccountStorage> {
    SecureStoreMigrationManager::new(
        config.base_data_directory.join(LEGACY_MNEMONIC_FILE_NAME),
        mnemonic_storage,
        account_storage,
    )
}
