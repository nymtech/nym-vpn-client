// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use nym_vpn_store::{
    keys::{
        DeviceKeys, KeyStore,
        persistence::{DeviceKeysPaths, OnDiskKeysError},
    },
    mnemonic::{Mnemonic, MnemonicStorage, on_disk::OnDiskMnemonicStorageError},
    stats::{StatsStorage, on_disk::OnDiskStatsStorageError},
};

mod helpers;

const MNEMONIC_FILE_NAME: &str = "mnemonic.json";
const STATS_DB_FILE_NAME: &str = "stats.db";

#[derive(Debug, thiserror::Error)]
pub enum VpnClientOnDiskStorageError {
    #[error("Statistics storage error : {0}")]
    Sqlx(#[from] OnDiskStatsStorageError),
}
pub struct VpnClientOnDiskStorage {
    key_store: nym_vpn_store::keys::persistence::OnDiskKeys,
    mnemonic_storage: nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorage,
    stats_storage: nym_vpn_store::stats::on_disk::OnDiskStatsStorage,
}

impl VpnClientOnDiskStorage {
    pub fn new<P: AsRef<Path>>(base_data_directory: P) -> Self {
        let device_key_paths = DeviceKeysPaths::new(&base_data_directory);
        let key_store = nym_vpn_store::keys::persistence::OnDiskKeys::new(device_key_paths);

        let mnemonic_storage_path = base_data_directory.as_ref().join(MNEMONIC_FILE_NAME);
        let mnemonic_storage =
            nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorage::new(mnemonic_storage_path);

        let stats_storage = nym_vpn_store::stats::on_disk::OnDiskStatsStorage::new(
            base_data_directory.as_ref().join(STATS_DB_FILE_NAME),
        );

        VpnClientOnDiskStorage {
            key_store,
            mnemonic_storage,
            stats_storage,
        }
    }
    // SW I hate this
    pub async fn init_sqlite_storage(&mut self) -> Result<(), VpnClientOnDiskStorageError> {
        Ok(self.stats_storage.init().await?)
    }
}

impl nym_vpn_store::VpnStorage for VpnClientOnDiskStorage {}

#[async_trait::async_trait]
impl KeyStore for VpnClientOnDiskStorage {
    type StorageError = OnDiskKeysError;

    async fn load_keys(&self) -> Result<DeviceKeys, Self::StorageError> {
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

    async fn load_mnemonic(&self) -> Result<Mnemonic, Self::StorageError> {
        self.mnemonic_storage.load_mnemonic().await
    }

    async fn store_mnemonic(&self, mnemonic: Mnemonic) -> Result<(), Self::StorageError> {
        self.mnemonic_storage.store_mnemonic(mnemonic).await
    }

    async fn remove_mnemonic(&self) -> Result<(), Self::StorageError> {
        self.mnemonic_storage.remove_mnemonic().await
    }
}

#[async_trait::async_trait]
impl StatsStorage for VpnClientOnDiskStorage {
    type StorageError = OnDiskStatsStorageError;

    async fn maybe_init_and_load_stats_seed(&self) -> Result<String, Self::StorageError> {
        self.stats_storage.maybe_init_and_load_stats_seed().await
    }

    async fn reset_stats_seed(&self) -> Result<String, Self::StorageError> {
        self.stats_storage.reset_stats_seed().await
    }

    async fn remove_stats_seed(&self) -> Result<(), Self::StorageError> {
        self.stats_storage.remove_stats_seed().await
    }
}
