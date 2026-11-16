// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;
#[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
use std::sync::Arc;

#[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
use nym_secure_enclave::SecureEnclave;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "android")))]
use nym_vpn_store::store_ciphers::noop_cipher::NoopStoreCipher;
#[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
use nym_vpn_store::store_ciphers::secure_enclave_cipher::{
    SecureEnclaveCipher, SecureEnclaveCipherOptions,
};
use nym_vpn_store::{
    account::{
        AccountInformationStorage, StoredDeviceState,
        on_disk::{OnDiskAccountStorage, OnDiskAccountStorageError},
    },
    database_key::{DatabaseKey, DatabaseKeyStorageError, OnDiskDatabaseKeyStorage},
    keys::device::{DeviceKeyStore, DeviceKeys, DeviceKeysPaths, OnDiskKeys, OnDiskKeysError},
    mnemonic::{
        MnemonicStorage, StoredMnemonic,
        on_disk::{OnDiskMnemonicStorage, OnDiskMnemonicStorageError},
    },
    secure_store_migration::SecureStoreMigrationManager,
};

const MNEMONIC_FILE_NAME: &str = "mnemonic.bin";
const ACCOUNT_INFO_FILE_NAME: &str = "account.bin";
const DATABASE_KEY_FILE_NAME: &str = "encryption.key";

/// Plaintext files replaced by the encrypted stores
const LEGACY_MNEMONIC_FILE_NAME: &str = "mnemonic.json";
const LEGACY_ACCOUNT_SUMMARY_FILE_NAME: &str = "account_summary.json";

#[cfg(target_os = "android")]
const MIGRATION_UNLOCK_REASON: &str =
    "Unlock your account credentials to perform migration from plain to encrypted account storage";

/// Key used for mnemonic encryption (checked user presence)
#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios"))]
const MNEMONIC_SE_KEY: &str = "mnemonic-key";

/// Key used for encryption of all other files (unattended)
#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios"))]
const DEVICE_SE_KEY: &str = "device-key";

pub struct VpnClientOnDiskStorage {
    key_store: nym_vpn_store::keys::device::OnDiskKeys,
    mnemonic_storage: nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorage,
    account_storage: nym_vpn_store::account::on_disk::OnDiskAccountStorage,
    database_key: DatabaseKey,
}

impl VpnClientOnDiskStorage {
    /// Creates the storage, loading the database key or creating it on first use.
    pub async fn new(
        base_data_directory: impl AsRef<Path>,
        #[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
        secure_enclave: Arc<SecureEnclave>,
    ) -> Result<Self, DatabaseKeyStorageError> {
        let device_key_paths = DeviceKeysPaths::new(&base_data_directory);

        let mnemonic_storage_path = base_data_directory.as_ref().join(MNEMONIC_FILE_NAME);

        #[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
        let mnemonic_storage_cipher = SecureEnclaveCipher::new(
            secure_enclave.clone(),
            SecureEnclaveCipherOptions {
                key_alias: MNEMONIC_SE_KEY.to_owned(),
                user_presence: if cfg!(any(target_os = "ios", target_os = "android")) {
                    // iOS (UI app) and Android can use attended key
                    // todo: ensure that iOS (network extension) does not attempt to access mnemonic
                    true
                } else {
                    // on macOS mnemonic key cannot require biometric prompt since
                    // nym-vpnd runs as root.
                    false
                },
            },
        );
        #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "android")))]
        let mnemonic_storage_cipher = NoopStoreCipher::default();
        let mnemonic_storage =
            OnDiskMnemonicStorage::new(mnemonic_storage_path, mnemonic_storage_cipher);

        #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "android")))]
        let account_storage_cipher = NoopStoreCipher::default();
        #[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
        let account_storage_cipher = SecureEnclaveCipher::new(
            secure_enclave.clone(),
            SecureEnclaveCipherOptions {
                key_alias: DEVICE_SE_KEY.to_owned(),
                user_presence: false,
            },
        );

        let account_storage_path = base_data_directory.as_ref().join(ACCOUNT_INFO_FILE_NAME);
        let account_storage =
            OnDiskAccountStorage::new(account_storage_path, account_storage_cipher);

        #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "android")))]
        let database_key_cipher = NoopStoreCipher::default();
        #[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
        let database_key_cipher = SecureEnclaveCipher::new(
            secure_enclave.clone(),
            SecureEnclaveCipherOptions {
                key_alias: DEVICE_SE_KEY.to_owned(),
                user_presence: false,
            },
        );

        let database_key_path = base_data_directory.as_ref().join(DATABASE_KEY_FILE_NAME);
        let database_key = OnDiskDatabaseKeyStorage::new(database_key_path, database_key_cipher)
            .load_or_create()
            .await?;
        let key_store = OnDiskKeys::new(device_key_paths, &database_key);

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
    type StorageError = OnDiskKeysError;

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
    type StorageError = OnDiskMnemonicStorageError;

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
    type StorageError = OnDiskAccountStorageError;

    async fn load_account(&self) -> Result<Option<StoredDeviceState>, Self::StorageError> {
        self.account_storage.load_account().await
    }

    async fn store_account(&self, state: StoredDeviceState) -> Result<(), Self::StorageError> {
        self.account_storage.store_account(state).await
    }

    async fn remove_account(&self) -> Result<(), Self::StorageError> {
        self.account_storage.remove_account().await
    }
}
