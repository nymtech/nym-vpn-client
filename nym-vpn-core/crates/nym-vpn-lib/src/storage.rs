// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;
#[cfg(target_os = "android")]
use std::sync::Arc;

#[cfg(target_os = "android")]
use nym_secure_enclave::SecureEnclave;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use nym_secure_enclave::{Keychain, KeychainAccess};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use nym_vpn_store::database_key::KeychainDatabaseKeyStorage;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
use nym_vpn_store::database_key::OnDiskDatabaseKeyStorage;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "android")))]
use nym_vpn_store::store_ciphers::noop_cipher::NoopStoreCipher;
#[cfg(target_os = "android")]
use nym_vpn_store::store_ciphers::secure_enclave_cipher::{
    SecureEnclaveCipher, SecureEnclaveCipherOptions,
};
use nym_vpn_store::{
    account::{AccountInformationStorage, StoredAccountInformation},
    database_key::{DatabaseKey, DatabaseKeyStorageError},
    keys::device::{DeviceKeyStore, DeviceKeys, DeviceKeysPaths},
    mnemonic::{MnemonicStorage, StoredMnemonic},
    secure_store_migration::SecureStoreMigrationManager,
};

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
const MNEMONIC_FILE_NAME: &str = "mnemonic.bin";
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
const ACCOUNT_INFO_FILE_NAME: &str = "account.bin";
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
const DATABASE_KEY_FILE_NAME: &str = "encryption.key";

/// Keychain access group shared by the iOS app and its network extension. On macOS, keys stay in
/// the default access group of the daemon.
#[cfg(target_os = "ios")]
pub const KEYCHAIN_ACCESS_GROUP: Option<&str> = Some("group.net.nymtech.vpn");
#[cfg(target_os = "macos")]
pub const KEYCHAIN_ACCESS_GROUP: Option<&str> = None;

/// Keychain service of the database key, stored per network
#[cfg(any(target_os = "ios", target_os = "macos"))]
const DATABASE_KEY_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.database-key";

/// Keychain service of the mnemonic, stored per network
#[cfg(any(target_os = "ios", target_os = "macos"))]
const MNEMONIC_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.mnemonic";

/// Keychain service of the account information, stored per network
#[cfg(any(target_os = "ios", target_os = "macos"))]
const ACCOUNT_INFO_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.account-info";

/// Keychain service of the device keys, stored per network
#[cfg(any(target_os = "ios", target_os = "macos"))]
const DEVICE_KEYS_KEYCHAIN_SERVICE: &str = "net.nymtech.vpn.device-keys";

#[cfg(any(target_os = "ios", target_os = "macos"))]
type PlatformMnemonicStorage = nym_vpn_store::mnemonic::keychain::KeychainMnemonicStorage;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
type PlatformMnemonicStorage = nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorage;

#[cfg(any(target_os = "ios", target_os = "macos"))]
type PlatformAccountStorage = nym_vpn_store::account::keychain::KeychainAccountStorage;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
type PlatformAccountStorage = nym_vpn_store::account::on_disk::OnDiskAccountStorage;

#[cfg(any(target_os = "ios", target_os = "macos"))]
type PlatformDeviceKeys = nym_vpn_store::keys::device::KeychainKeys;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
type PlatformDeviceKeys = nym_vpn_store::keys::device::OnDiskKeys;

/// Plaintext files replaced by the encrypted stores
const LEGACY_MNEMONIC_FILE_NAME: &str = "mnemonic.json";
const LEGACY_ACCOUNT_SUMMARY_FILE_NAME: &str = "account_summary.json";

#[cfg(target_os = "android")]
const MIGRATION_UNLOCK_REASON: &str =
    "Unlock your account credentials to perform migration from plain to encrypted account storage";

/// Key used for mnemonic encryption (checked user presence)
#[cfg(target_os = "android")]
const MNEMONIC_SE_KEY: &str = "mnemonic-key";

/// Key used for encryption of all other files (unattended)
#[cfg(target_os = "android")]
const DEVICE_SE_KEY: &str = "device-key";

pub struct VpnClientOnDiskStorage {
    key_store: PlatformDeviceKeys,
    mnemonic_storage: PlatformMnemonicStorage,
    account_storage: PlatformAccountStorage,
    database_key: DatabaseKey,
}

impl VpnClientOnDiskStorage {
    /// Creates the storage, loading the database key or creating it on first use.
    pub async fn new(
        base_data_directory: impl AsRef<Path>,
        #[cfg_attr(
            not(any(target_os = "ios", target_os = "macos")),
            expect(unused_variables)
        )]
        network_name: &str,
        #[cfg(any(target_os = "ios", target_os = "macos"))] keychain: Keychain,
        #[cfg(target_os = "android")] secure_enclave: Arc<SecureEnclave>,
    ) -> Result<Self, DatabaseKeyStorageError> {
        let device_key_paths = DeviceKeysPaths::new(&base_data_directory);

        // The iOS app can prompt for user presence, while nym-vpnd on macOS runs as root.
        // todo: ensure that iOS (network extension) does not attempt to access mnemonic
        #[cfg(target_os = "ios")]
        let mnemonic_access = KeychainAccess::UserPresence;
        #[cfg(target_os = "macos")]
        let mnemonic_access = KeychainAccess::AfterFirstUnlock;
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        let mnemonic_storage = PlatformMnemonicStorage::new(keychain.item(
            MNEMONIC_KEYCHAIN_SERVICE,
            network_name,
            mnemonic_access,
        ));
        #[cfg(not(any(target_os = "ios", target_os = "macos")))]
        let mnemonic_storage = {
            #[cfg(target_os = "android")]
            let mnemonic_storage_cipher = SecureEnclaveCipher::new(
                secure_enclave.clone(),
                SecureEnclaveCipherOptions {
                    key_alias: MNEMONIC_SE_KEY.to_owned(),
                    user_presence: true,
                },
            );
            #[cfg(not(target_os = "android"))]
            let mnemonic_storage_cipher = NoopStoreCipher::default();

            let mnemonic_storage_path = base_data_directory.as_ref().join(MNEMONIC_FILE_NAME);
            PlatformMnemonicStorage::new(mnemonic_storage_path, mnemonic_storage_cipher)
        };

        #[cfg(any(target_os = "ios", target_os = "macos"))]
        let account_storage = PlatformAccountStorage::new(keychain.item(
            ACCOUNT_INFO_KEYCHAIN_SERVICE,
            network_name,
            KeychainAccess::AfterFirstUnlock,
        ));
        #[cfg(not(any(target_os = "ios", target_os = "macos")))]
        let account_storage = {
            #[cfg(not(target_os = "android"))]
            let account_storage_cipher = NoopStoreCipher::default();
            #[cfg(target_os = "android")]
            let account_storage_cipher = SecureEnclaveCipher::new(
                secure_enclave.clone(),
                SecureEnclaveCipherOptions {
                    key_alias: DEVICE_SE_KEY.to_owned(),
                    user_presence: false,
                },
            );

            let account_storage_path = base_data_directory.as_ref().join(ACCOUNT_INFO_FILE_NAME);
            PlatformAccountStorage::new(account_storage_path, account_storage_cipher)
        };

        #[cfg(any(target_os = "ios", target_os = "macos"))]
        let database_key = KeychainDatabaseKeyStorage::new(keychain.item(
            DATABASE_KEY_KEYCHAIN_SERVICE,
            network_name,
            KeychainAccess::AfterFirstUnlock,
        ))
        .load_or_create()
        .await?;
        #[cfg(not(any(target_os = "ios", target_os = "macos")))]
        let database_key = {
            #[cfg(not(target_os = "android"))]
            let database_key_cipher = NoopStoreCipher::default();
            #[cfg(target_os = "android")]
            let database_key_cipher = SecureEnclaveCipher::new(
                secure_enclave.clone(),
                SecureEnclaveCipherOptions {
                    key_alias: DEVICE_SE_KEY.to_owned(),
                    user_presence: false,
                },
            );

            let database_key_path = base_data_directory.as_ref().join(DATABASE_KEY_FILE_NAME);
            OnDiskDatabaseKeyStorage::new(database_key_path, database_key_cipher)
                .load_or_create()
                .await?
        };

        #[cfg(any(target_os = "ios", target_os = "macos"))]
        let key_store = PlatformDeviceKeys::new(keychain.item(
            DEVICE_KEYS_KEYCHAIN_SERVICE,
            network_name,
            KeychainAccess::AfterFirstUnlock,
        ))
        .with_plaintext_migration(device_key_paths);
        #[cfg(not(any(target_os = "ios", target_os = "macos")))]
        let key_store = PlatformDeviceKeys::new(device_key_paths, &database_key);

        let migration = SecureStoreMigrationManager::new(
            base_data_directory.as_ref().join(LEGACY_MNEMONIC_FILE_NAME),
            base_data_directory
                .as_ref()
                .join(LEGACY_ACCOUNT_SUMMARY_FILE_NAME),
            &mnemonic_storage,
            &account_storage,
            #[cfg(target_os = "android")]
            secure_enclave,
            #[cfg(target_os = "android")]
            MIGRATION_UNLOCK_REASON,
        );
        // Plaintext files are kept on failure, so the migration is retried on next start.
        if let Err(err) = migration.migrate().await {
            tracing::error!("Failed to migrate to secure store: {err}");
        }

        Ok(VpnClientOnDiskStorage {
            key_store,
            mnemonic_storage,
            account_storage,
            database_key,
        })
    }

    /// Key used to encrypt the on-disk databases.
    pub fn database_key(&self) -> &DatabaseKey {
        &self.database_key
    }
}

impl nym_vpn_store::VpnStorage for VpnClientOnDiskStorage {}

#[async_trait::async_trait]
impl DeviceKeyStore for VpnClientOnDiskStorage {
    type StorageError = <PlatformDeviceKeys as DeviceKeyStore>::StorageError;

    async fn load_keys(&self) -> Result<Option<DeviceKeys>, Self::StorageError> {
        self.key_store.load_keys().await
    }

    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), Self::StorageError> {
        self.key_store.store_keys(keys).await
    }

    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError> {
        self.key_store.init_keys(seed).await
    }

    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Self::StorageError> {
        self.key_store.reset_keys(seed).await
    }

    async fn remove_keys(&self) -> Result<(), Self::StorageError> {
        self.key_store.remove_keys().await
    }
}

#[async_trait::async_trait]
impl MnemonicStorage for VpnClientOnDiskStorage {
    type StorageError = <PlatformMnemonicStorage as MnemonicStorage>::StorageError;

    async fn load_mnemonic(&self) -> Result<Option<StoredMnemonic>, Self::StorageError> {
        self.mnemonic_storage.load_mnemonic().await
    }

    async fn store_mnemonic(&self, mnemonic: StoredMnemonic) -> Result<(), Self::StorageError> {
        self.mnemonic_storage.store_mnemonic(mnemonic).await
    }

    async fn remove_mnemonic(&self) -> Result<(), Self::StorageError> {
        self.mnemonic_storage.remove_mnemonic().await
    }

    async fn is_mnemonic_stored(&self) -> Result<bool, Self::StorageError> {
        self.mnemonic_storage.is_mnemonic_stored().await
    }
}

#[async_trait::async_trait]
impl AccountInformationStorage for VpnClientOnDiskStorage {
    type StorageError = <PlatformAccountStorage as AccountInformationStorage>::StorageError;

    async fn load_account(&self) -> Result<Option<StoredAccountInformation>, Self::StorageError> {
        self.account_storage.load_account().await
    }

    async fn store_account(
        &self,
        state: StoredAccountInformation,
    ) -> Result<(), Self::StorageError> {
        self.account_storage.store_account(state).await
    }

    async fn remove_account(&self) -> Result<(), Self::StorageError> {
        self.account_storage.remove_account().await
    }
}
