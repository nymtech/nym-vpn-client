// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use nym_vpn_lib_types::{StorableAccount, VpnAccountSummary};
use nym_vpn_store::{
    account::{AccountInformationStorage, on_disk::OnDiskMnemonicStorageError},
    account_summary::{AccountSummaryStorage, on_disk::OnDiskAccountSummaryStorageError},
    keys::device::{DeviceKeyStore, DeviceKeys, DeviceKeysPaths, OnDiskKeysError},
};

const MNEMONIC_FILE_NAME: &str = "mnemonic.json";
pub const ACCOUNT_SUMMARY_FILE_NAME: &str = "account_summary.json";

pub struct VpnClientOnDiskStorage {
    key_store: nym_vpn_store::keys::device::OnDiskKeys,
    account_storage: nym_vpn_store::account::on_disk::OnDiskAccountStorage,
    summary_storage: nym_vpn_store::account_summary::on_disk::OnDiskAccountSummaryStorage,
}

impl VpnClientOnDiskStorage {
    pub fn new<P: AsRef<Path>>(base_data_directory: P) -> Self {
        let device_key_paths = DeviceKeysPaths::new(&base_data_directory);
        let key_store = nym_vpn_store::keys::device::OnDiskKeys::new(device_key_paths);

        let mnemonic_storage_path = base_data_directory.as_ref().join(MNEMONIC_FILE_NAME);
        let mnemonic_storage =
            nym_vpn_store::account::on_disk::OnDiskAccountStorage::new(mnemonic_storage_path);

        let summary_storage_path = base_data_directory.as_ref().join(ACCOUNT_SUMMARY_FILE_NAME);
        let summary_storage =
            nym_vpn_store::account_summary::on_disk::OnDiskAccountSummaryStorage::new(
                summary_storage_path,
            );

        VpnClientOnDiskStorage {
            key_store,
            account_storage: mnemonic_storage,
            summary_storage,
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

#[async_trait::async_trait]
impl AccountSummaryStorage for VpnClientOnDiskStorage {
    type StorageError = OnDiskAccountSummaryStorageError;

    async fn load_summary(&self) -> Result<Option<VpnAccountSummary>, Self::StorageError> {
        self.summary_storage.load_summary().await
    }

    async fn store_summary(&self, account: VpnAccountSummary) -> Result<(), Self::StorageError> {
        self.summary_storage.store_summary(account).await
    }

    async fn remove_summary(&self) -> Result<(), Self::StorageError> {
        self.summary_storage.remove_summary().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nym_vpn_lib_types::{
        NymVpnSubscription, NymVpnSubscriptionKind, NymVpnSubscriptionStatus, Subscription,
        VpnAccountStatus,
    };
    use nym_vpn_store::account_summary::on_disk::OnDiskAccountSummaryStorage;
    use time::OffsetDateTime;

    fn sample_summary() -> VpnAccountSummary {
        let now = OffsetDateTime::now_utc();
        VpnAccountSummary {
            traffic_used_gb: 0,
            traffic_limit_gb: 2000,
            traffic_reset_time: None,
            fair_usage_data_unavailable: false,
            account_addr: "n1test".into(),
            canonical_account_addr: None,
            auth_methods: vec![],
            account_mode: None,
            subscription: Some(Subscription {
                status: NymVpnSubscriptionStatus::Active,
                subscription: NymVpnSubscription {
                    created_on_utc: "2024-01-01T00:00:00Z".into(),
                    last_updated_utc: "2024-01-01T00:00:00Z".into(),
                    id: "sub".into(),
                    valid_from_utc: now.unix_timestamp() - 86_400,
                    valid_until_utc: now.unix_timestamp() + 365 * 86_400,
                    status: "active".into(),
                    kind: NymVpnSubscriptionKind::OneMonth,
                    is_recurring: false,
                },
            }),
            is_subscription_stacked: false,
            account_status: VpnAccountStatus::Active,
            is_device_active: true,
            remaining_devices: 1,
            time_synced: true,
            stale: false,
            last_synced_utc: now,
        }
    }

    #[tokio::test]
    async fn client_storage_summary_path_matches_on_disk_reader() {
        let dir = tempfile::tempdir().expect("tempdir");
        let summary = sample_summary();
        let storage = VpnClientOnDiskStorage::new(dir.path());

        storage
            .store_summary(summary.clone())
            .await
            .expect("store via client storage");

        let path = dir.path().join(ACCOUNT_SUMMARY_FILE_NAME);
        assert!(path.is_file(), "summary must land at canonical path");

        let direct = OnDiskAccountSummaryStorage::new(path);
        let loaded = direct
            .load_summary()
            .await
            .expect("load")
            .expect("summary present");
        assert_eq!(loaded.account_addr, summary.account_addr);
        assert_eq!(loaded.is_device_active, summary.is_device_active);
    }
}
