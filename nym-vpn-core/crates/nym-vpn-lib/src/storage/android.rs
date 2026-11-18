// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::{
    database_key::{DatabaseKeyStorageError, on_disk::OnDiskDatabaseKeyStorage},
    keys::device::DeviceKeysPaths,
    secure_store_migration::SecureStoreMigrationManager,
    store_ciphers::secure_enclave_cipher::{SecureEnclaveCipher, SecureEnclaveCipherOptions},
};

use super::{LEGACY_MNEMONIC_FILE_NAME, PlatformStores, StorageConfig};

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

/// Stores secrets in files encrypted with keys held by the Android keystore.
pub async fn create_stores(
    config: &StorageConfig,
) -> Result<PlatformStores, DatabaseKeyStorageError> {
    let base_data_directory = &config.base_data_directory;
    let cipher = |key_alias: &str, user_presence| {
        SecureEnclaveCipher::new(
            config.secure_enclave.clone(),
            SecureEnclaveCipherOptions {
                key_alias: key_alias.to_owned(),
                user_presence,
            },
        )
    };

    let database_key = OnDiskDatabaseKeyStorage::new(
        base_data_directory.join(DATABASE_KEY_FILE_NAME),
        cipher(DEVICE_SE_KEY, false),
    )
    .load_or_create()
    .await?;

    Ok(PlatformStores {
        mnemonic_storage: PlatformMnemonicStorage::new(
            base_data_directory.join(MNEMONIC_FILE_NAME),
            cipher(MNEMONIC_SE_KEY, true),
        ),
        account_storage: PlatformAccountStorage::new(
            base_data_directory.join(ACCOUNT_INFO_FILE_NAME),
            cipher(DEVICE_SE_KEY, false),
        ),
        key_store: PlatformDeviceKeys::new(
            DeviceKeysPaths::new(base_data_directory),
            &database_key,
        ),
        database_key,
    })
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
