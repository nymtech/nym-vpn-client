// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_secure_enclave::KeychainAccess;
use nym_vpn_store::{
    database_key::{DatabaseKeyStorageError, keychain::KeychainDatabaseKeyStorage},
    keys::device::DeviceKeysPaths,
    secure_store_migration::SecureStoreMigrationManager,
};

use super::{LEGACY_MNEMONIC_FILE_NAME, PlatformStores, StorageConfig};

pub type PlatformMnemonicStorage = nym_vpn_store::mnemonic::keychain::KeychainMnemonicStorage;
pub type PlatformAccountStorage = nym_vpn_store::account::keychain::KeychainAccountStorage;
pub type PlatformDeviceKeys = nym_vpn_store::keys::device::KeychainKeys;

/// Keychain access group shared by the iOS app and its network extension.
#[cfg(target_os = "ios")]
pub const KEYCHAIN_ACCESS_GROUP: Option<&str> = Some("group.net.nymtech.vpn");
/// Keys stay in the default access group of the daemon.
#[cfg(target_os = "macos")]
pub const KEYCHAIN_ACCESS_GROUP: Option<&str> = None;

/// The iOS app can prompt for user presence, while nym-vpnd on macOS runs as root.
// todo: ensure that iOS (network extension) does not attempt to access mnemonic
#[cfg(target_os = "ios")]
const MNEMONIC_ACCESS: KeychainAccess = KeychainAccess::UserPresence;
#[cfg(target_os = "macos")]
const MNEMONIC_ACCESS: KeychainAccess = KeychainAccess::AfterFirstUnlock;

/// Keychain services of the items, each stored per network.
const DATABASE_KEY_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.database-key";
const MNEMONIC_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.mnemonic";
const ACCOUNT_INFO_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.account-info";
const DEVICE_KEYS_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.device-keys";

/// Stores secrets in the keychain, moving plaintext device keys found on disk there.
pub async fn create_stores(
    config: &StorageConfig,
) -> Result<PlatformStores, DatabaseKeyStorageError> {
    let item = |service, access| config.keychain.item(service, &config.network_name, access);

    let database_key = KeychainDatabaseKeyStorage::new(item(
        DATABASE_KEY_KEYCHAIN_SERVICE,
        KeychainAccess::AfterFirstUnlock,
    ))
    .load_or_create()
    .await?;

    Ok(PlatformStores {
        mnemonic_storage: PlatformMnemonicStorage::new(item(
            MNEMONIC_KEYCHAIN_SERVICE,
            MNEMONIC_ACCESS,
        )),
        account_storage: PlatformAccountStorage::new(item(
            ACCOUNT_INFO_KEYCHAIN_SERVICE,
            KeychainAccess::AfterFirstUnlock,
        )),
        key_store: PlatformDeviceKeys::new(item(
            DEVICE_KEYS_KEYCHAIN_SERVICE,
            KeychainAccess::AfterFirstUnlock,
        ))
        .with_plaintext_migration(DeviceKeysPaths::new(&config.base_data_directory)),
        database_key,
    })
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
