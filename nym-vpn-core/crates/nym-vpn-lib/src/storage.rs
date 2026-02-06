// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use nym_vpn_store::{
    account::{AccountInformationStorage, on_disk::OnDiskMnemonicStorageError},
    keys::device::{DeviceKeyStore, DeviceKeys, DeviceKeysPaths, OnDiskKeysError},
    types::StorableAccount,
};

const MNEMONIC_FILE_NAME: &str = "mnemonic.json";

pub struct VpnClientOnDiskStorage {
    key_store: nym_vpn_store::keys::device::OnDiskKeys,
    account_storage: nym_vpn_store::account::on_disk::OnDiskAccountStorage,
}

impl VpnClientOnDiskStorage {
    pub fn new<P: AsRef<Path>>(base_data_directory: P) -> Self {
        let device_key_paths = DeviceKeysPaths::new(&base_data_directory);
        let key_store = nym_vpn_store::keys::device::OnDiskKeys::new(device_key_paths);

        let mnemonic_storage_path = base_data_directory.as_ref().join(MNEMONIC_FILE_NAME);
        let mnemonic_storage =
            nym_vpn_store::account::on_disk::OnDiskAccountStorage::new(mnemonic_storage_path);

        VpnClientOnDiskStorage {
            key_store,
            account_storage: mnemonic_storage,
        }
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
impl AccountInformationStorage for VpnClientOnDiskStorage {
    type StorageError = OnDiskMnemonicStorageError;

    async fn load_account(&self) -> Result<Option<StorableAccount>, Self::StorageError> {
        self.account_storage.load_account().await
    }

    async fn store_account(&self, account: StorableAccount) -> Result<(), Self::StorageError> {
        self.account_storage.store_account(account).await
    }

    async fn remove_account(&self) -> Result<(), Self::StorageError> {
        self.account_storage.remove_account().await
    }
}
