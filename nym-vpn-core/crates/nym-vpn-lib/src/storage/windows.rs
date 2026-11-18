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

const MNEMONIC_FILE_NAME: &str = "mnemonic.bin";
const ACCOUNT_INFO_FILE_NAME: &str = "account.bin";
const DATABASE_KEY_FILE_NAME: &str = "encryption.key";

/// Name of the TPM key, which is shared by all programs of the user.
const DEVICE_SE_KEY: &str = "net.nymtech.vpn.device-key";

/// Cipher for all files, backed by the TPM or by DPAPI if no TPM is available.
fn device_cipher(config: &StorageConfig) -> SecureEnclaveCipher {
    SecureEnclaveCipher::new(
        config.secure_enclave.clone(),
        SecureEnclaveCipherOptions {
            key_alias: DEVICE_SE_KEY.to_owned(),
            user_presence: false,
        },
    )
}

/// Loads the database key, encrypted with a key held by the TPM.
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
        device_cipher(config),
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
