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
    device_state::{
        StoredDeviceState, DeviceStateStorage,
        on_disk::{OnDiskDeviceStateStorage, OnDiskDeviceStateStorageError},
    },
    keys::device::{DeviceKeyStore, DeviceKeys, DeviceKeysPaths, OnDiskKeys, OnDiskKeysError},
    mnemonic::{
        MnemonicStorage, StoredMnemonic,
        on_disk::{OnDiskMnemonicStorage, OnDiskMnemonicStorageError},
    },
};

const MNEMONIC_FILE_NAME: &str = "mnemonic.bin";
const DEVICE_STATE_FILE_NAME: &str = "device_state.bin";

/// Key used for mnemonic encryption (checked user presence)
#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios"))]
const MNEMONIC_SE_KEY: &str = "mnemonic-key";

/// Key used for encryption of all other files (unattended)
#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios"))]
const DEVICE_SE_KEY: &str = "device-key";

pub struct VpnClientOnDiskStorage {
    key_store: nym_vpn_store::keys::device::OnDiskKeys,
    mnemonic_storage: nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorage,
    device_state_storage: nym_vpn_store::device_state::on_disk::OnDiskDeviceStateStorage,
}

impl VpnClientOnDiskStorage {
    pub fn new(
        base_data_directory: impl AsRef<Path>,
        #[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
        secure_enclave: Arc<SecureEnclave>,
    ) -> Self {
        let device_key_paths = DeviceKeysPaths::new(&base_data_directory);
        let key_store = OnDiskKeys::new(device_key_paths);

        let mnemonic_storage_path = base_data_directory.as_ref().join(MNEMONIC_FILE_NAME);

        #[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
        let mnemonic_storage_cipher = SecureEnclaveCipher::new(
            secure_enclave.clone(),
            SecureEnclaveCipherOptions {
                key_alias: MNEMONIC_SE_KEY.to_owned(),
                user_presence: true,
            },
        );
        #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "android")))]
        let mnemonic_storage_cipher = NoopStoreCipher::default();
        let mnemonic_storage =
            OnDiskMnemonicStorage::new(mnemonic_storage_path, mnemonic_storage_cipher);

        #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "android")))]
        let device_state_storage_cipher = NoopStoreCipher::default();
        #[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
        let device_state_storage_cipher = SecureEnclaveCipher::new(
            secure_enclave,
            SecureEnclaveCipherOptions {
                key_alias: DEVICE_SE_KEY.to_owned(),
                user_presence: false,
            },
        );

        let device_state_storage_path = base_data_directory.as_ref().join(DEVICE_STATE_FILE_NAME);
        let device_state_storage =
            OnDiskDeviceStateStorage::new(device_state_storage_path, device_state_storage_cipher);

        VpnClientOnDiskStorage {
            key_store,
            mnemonic_storage,
            device_state_storage,
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
impl DeviceStateStorage for VpnClientOnDiskStorage {
    type StorageError = OnDiskDeviceStateStorageError;

    async fn load_device_state(&self) -> Result<StoredDeviceState, Self::StorageError> {
        self.device_state_storage.load_device_state().await
    }

    async fn store_device_state(&self, state: StoredDeviceState) -> Result<(), Self::StorageError> {
        self.device_state_storage.store_device_state(state).await
    }

    async fn remove_device_state(&self) -> Result<(), Self::StorageError> {
        self.device_state_storage.remove_device_state().await
    }
}
