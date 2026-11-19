// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_secure_enclave::{KeychainAccess, KeychainItem};
use nym_vpn_store::{
    keys::device::DeviceKeysPaths,
    master_key::{KeySecret, MasterKey, MasterKeyStorageError, keychain::KeychainMasterKeyStorage},
};

use super::StorageConfig;

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
const MASTER_KEY_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.master-key";
const MNEMONIC_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.mnemonic";
const ACCOUNT_INFO_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.account-info";
const DEVICE_KEYS_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.device-keys";

fn keychain_item(config: &StorageConfig, service: &str, access: KeychainAccess) -> KeychainItem {
    config.keychain.item(service, &config.network_name, access)
}

pub async fn master_key(config: &StorageConfig) -> Result<MasterKey, MasterKeyStorageError> {
    KeychainMasterKeyStorage::new(keychain_item(
        config,
        MASTER_KEY_KEYCHAIN_SERVICE,
        KeychainAccess::AfterFirstUnlock,
    ))
    .load_or_create()
    .await
}

pub fn mnemonic_storage(config: &StorageConfig) -> PlatformMnemonicStorage {
    PlatformMnemonicStorage::new(keychain_item(
        config,
        MNEMONIC_KEYCHAIN_SERVICE,
        MNEMONIC_ACCESS,
    ))
}

pub fn account_storage(
    config: &StorageConfig,
    _master_key: &MasterKey,
) -> Result<PlatformAccountStorage, MasterKeyStorageError> {
    Ok(PlatformAccountStorage::new(keychain_item(
        config,
        ACCOUNT_INFO_KEYCHAIN_SERVICE,
        KeychainAccess::AfterFirstUnlock,
    )))
}

/// Moves plaintext device keys found on disk to the keychain.
pub fn device_keys(config: &StorageConfig, _key_secret: KeySecret) -> PlatformDeviceKeys {
    PlatformDeviceKeys::new(keychain_item(
        config,
        DEVICE_KEYS_KEYCHAIN_SERVICE,
        KeychainAccess::AfterFirstUnlock,
    ))
    .with_plaintext_migration(DeviceKeysPaths::new(&config.base_data_directory))
}
