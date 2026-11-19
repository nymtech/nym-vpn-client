// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
mod android;
#[cfg(any(target_os = "ios", target_os = "macos"))]
mod apple;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "android",
    target_os = "windows",
    target_os = "linux"
)))]
mod other;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
use self::windows as platform;
#[cfg(target_os = "android")]
use android as platform;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use apple as platform;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(not(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "android",
    target_os = "windows",
    target_os = "linux"
)))]
use other as platform;

use std::path::PathBuf;
#[cfg(any(target_os = "android", target_os = "windows", target_os = "linux"))]
use std::sync::Arc;

#[cfg(any(target_os = "ios", target_os = "macos"))]
use nym_secure_enclave::Keychain;
#[cfg(any(target_os = "android", target_os = "windows", target_os = "linux"))]
use nym_secure_enclave::SecureEnclave;
use nym_vpn_lib_types::VpnAccountSummary;
use nym_vpn_store::{
    account::{AccountInformationStorage, StoredAccountInformation},
    account_summary::{
        AccountSummaryStorage,
        on_disk::{OnDiskAccountSummaryStorage, OnDiskAccountSummaryStorageError},
    },
    keys::device::{DeviceKeyStore, DeviceKeys},
    master_key::{MasterKeyStorageError, StorageKeys},
    mnemonic::{MnemonicStorage, StoredMnemonic},
    secure_store_migration::SecureStoreMigrationManager,
};
use typed_builder::TypedBuilder;

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use apple::KEYCHAIN_ACCESS_GROUP;
use platform::{PlatformAccountStorage, PlatformDeviceKeys, PlatformMnemonicStorage};

const ACCOUNT_SUMMARY_FILE_NAME: &str = "account_summary.bin";

/// Plaintext files replaced by the encrypted stores
const LEGACY_MNEMONIC_FILE_NAME: &str = "mnemonic.json";
const LEGACY_ACCOUNT_SUMMARY_FILE_NAME: &str = "account_summary.json";

/// Configuration of [`VpnClientOnDiskStorage`].
///
/// Secrets are stored per `network_name` in `keychain` on Apple platforms, in files encrypted with
/// keys held by `secure_enclave` on Android, Windows and Linux, and in plain files elsewhere.
#[derive(TypedBuilder)]
pub struct StorageConfig {
    /// Directory of the network the storage belongs to.
    #[builder(setter(into))]
    base_data_directory: PathBuf,

    /// Account of the keychain items.
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[builder(setter(into))]
    network_name: String,

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    keychain: Keychain,

    #[cfg(any(target_os = "android", target_os = "windows", target_os = "linux"))]
    secure_enclave: Arc<SecureEnclave>,
}

pub struct VpnClientOnDiskStorage {
    key_store: PlatformDeviceKeys,
    mnemonic_storage: PlatformMnemonicStorage,
    account_storage: PlatformAccountStorage,
    summary_storage: OnDiskAccountSummaryStorage,
    storage_keys: StorageKeys,
}

impl VpnClientOnDiskStorage {
    /// Creates the storage, loading the master key or creating it on first use.
    pub async fn new(config: StorageConfig) -> Result<Self, MasterKeyStorageError> {
        let master_key = platform::master_key(&config).await?;
        let storage_keys = master_key.derive_storage_keys()?;
        let summary_cipher = master_key.derive_store_cipher()?;
        drop(master_key);
        let mnemonic_storage = platform::mnemonic_storage(&config);
        let account_storage = platform::account_storage(&config);
        let key_store = platform::device_keys(&config, storage_keys.key_secret.clone());
        let base_data_directory = &config.base_data_directory;

        let summary_storage = OnDiskAccountSummaryStorage::new(
            base_data_directory.join(ACCOUNT_SUMMARY_FILE_NAME),
            summary_cipher,
        )
        .with_legacy_plaintext(base_data_directory.join(LEGACY_ACCOUNT_SUMMARY_FILE_NAME));

        let migration = SecureStoreMigrationManager::builder()
            .mnemonic_json_path(base_data_directory.join(LEGACY_MNEMONIC_FILE_NAME))
            .mnemonic_storage(&mnemonic_storage)
            .account_storage(&account_storage);
        #[cfg(target_os = "android")]
        let migration = migration
            .secure_enclave(config.secure_enclave.clone())
            .unlock_reason(platform::MIGRATION_UNLOCK_REASON);
        let migration = migration.build();
        // Plaintext files are kept on failure, so the migration is retried on next start.
        if let Err(err) = migration.migrate().await {
            tracing::error!("Failed to migrate to secure store: {err}");
        }

        Ok(VpnClientOnDiskStorage {
            key_store,
            mnemonic_storage,
            account_storage,
            summary_storage,
            storage_keys,
        })
    }

    /// Keys encrypting the on-disk databases and private key files.
    pub fn storage_keys(&self) -> &StorageKeys {
        &self.storage_keys
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

#[async_trait::async_trait]
impl AccountSummaryStorage for VpnClientOnDiskStorage {
    type StorageError = OnDiskAccountSummaryStorageError;

    async fn load_summary(&self) -> Result<Option<VpnAccountSummary>, Self::StorageError> {
        self.summary_storage.load_summary().await
    }

    async fn store_summary(&self, summary: &VpnAccountSummary) -> Result<(), Self::StorageError> {
        self.summary_storage.store_summary(summary).await
    }

    async fn remove_summary(&self) -> Result<(), Self::StorageError> {
        self.summary_storage.remove_summary().await
    }
}
