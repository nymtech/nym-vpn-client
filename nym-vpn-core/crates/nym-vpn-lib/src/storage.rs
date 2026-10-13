// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

#[cfg(any(target_os = "ios", target_os = "android"))]
use futures::TryFutureExt;
#[cfg(any(target_os = "ios", target_os = "android"))]
use nym_secure_enclave::{SecureEnclave, SecureEnclaveError};
use nym_vpn_lib_types::{StorableAccount, VpnAccountSummary};
#[cfg(not(any(target_os = "ios", target_os = "android")))]
use nym_vpn_store::ciphers::noop_cipher::NoopStoreCipher;
#[cfg(any(target_os = "ios", target_os = "android"))]
use nym_vpn_store::ciphers::secure_enclave_cipher::SecureEnclaveCipher;
use nym_vpn_store::{
    account::{AccountInformationStorage, on_disk::OnDiskMnemonicStorageError},
    account_summary::{AccountSummaryStorage, on_disk::OnDiskAccountSummaryStorageError},
    keys::device::{DeviceKeyStore, DeviceKeys, DeviceKeysPaths, OnDiskKeysError},
};

const MNEMONIC_FILE_NAME: &str = "mnemonic.json";
const ACCOUNT_SUMMARY_FILE_NAME: &str = "account_summary.json";

#[cfg(any(target_os = "android", target_os = "ios"))]
const MNEMONIC_SE_KEY: &str = "mnemonic-key";

pub struct VpnClientOnDiskStorage {
    key_store: nym_vpn_store::keys::device::OnDiskKeys,
    account_storage: nym_vpn_store::account::on_disk::OnDiskAccountStorage,
    summary_storage: nym_vpn_store::account_summary::on_disk::OnDiskAccountSummaryStorage,
}

impl VpnClientOnDiskStorage {
    pub async fn new(
        base_data_directory: impl AsRef<Path>,
        #[cfg(any(target_os = "ios", target_os = "android"))] secure_enclave: SecureEnclave,
    ) -> Self {
        let device_key_paths = DeviceKeysPaths::new(&base_data_directory);
        let key_store = nym_vpn_store::keys::device::OnDiskKeys::new(device_key_paths);

        let mnemonic_storage_path = base_data_directory.as_ref().join(MNEMONIC_FILE_NAME);
        let mnemonic_storage_cipher = mnemonic_cipher(
            #[cfg(any(target_os = "ios", target_os = "android"))]
            secure_enclave,
        )
        .await;
        let mnemonic_storage = nym_vpn_store::account::on_disk::OnDiskAccountStorage::new(
            mnemonic_storage_path,
            mnemonic_storage_cipher,
        );

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

#[cfg(any(target_os = "android", target_os = "ios"))]
async fn mnemonic_cipher(secure_enclave: SecureEnclave) -> SecureEnclaveCipher {
    let se_cipher = SecureEnclaveCipher::new(async move {
        let alias = MNEMONIC_SE_KEY.to_owned();

        let se_key = secure_enclave
            .get_key(alias.clone())
            .or_else(|err| async {
                if err == SecureEnclaveError::KeyNotFound {
                    secure_enclave.create_key(alias).await
                } else {
                    Err(err)
                }
            })
            .await?;
    });

    Ok(se_cipher)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn mnemonic_cipher() -> NoopStoreCipher {
    NoopStoreCipher::default()
}
