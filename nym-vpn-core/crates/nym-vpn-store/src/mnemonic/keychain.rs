// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_secure_enclave::{KeychainError, KeychainItem};
use zeroize::Zeroizing;

use super::{MnemonicStorage, StoredMnemonic};

#[derive(Debug, thiserror::Error)]
pub enum KeychainMnemonicStorageError {
    #[error("mnemonic already stored")]
    MnemonicAlreadyStored,

    #[error("mnemonic keychain error")]
    Keychain(#[from] KeychainError),

    #[error("failed to serialize mnemonic")]
    Serialize(#[source] serde_json::Error),

    #[error("failed to deserialize mnemonic")]
    Deserialize(#[source] serde_json::Error),
}

pub type Result<T, E = KeychainMnemonicStorageError> = std::result::Result<T, E>;

/// Stores the mnemonic in the keychain.
pub struct KeychainMnemonicStorage {
    item: KeychainItem,
}

impl KeychainMnemonicStorage {
    pub fn new(item: KeychainItem) -> Self {
        Self { item }
    }
}

#[async_trait::async_trait]
impl MnemonicStorage for KeychainMnemonicStorage {
    type StorageError = KeychainMnemonicStorageError;

    async fn load_mnemonic(&self) -> Result<Option<StoredMnemonic>> {
        self.item
            .get_value()
            .await?
            .map(|value| {
                serde_json::from_slice::<StoredMnemonic>(&value)
                    .map_err(KeychainMnemonicStorageError::Deserialize)
            })
            .transpose()
    }

    async fn store_mnemonic(&self, mnemonic: StoredMnemonic) -> Result<()> {
        tracing::info!("Storing mnemonic in the keychain");

        let value = serde_json::to_vec(&mnemonic)
            .map(Zeroizing::new)
            .map_err(KeychainMnemonicStorageError::Serialize)?;

        if self.item.add(&value)? {
            Ok(())
        } else {
            Err(KeychainMnemonicStorageError::MnemonicAlreadyStored)
        }
    }

    async fn remove_mnemonic(&self) -> Result<()> {
        Ok(self.item.delete()?)
    }

    async fn is_mnemonic_stored(&self) -> Result<bool> {
        Ok(self.item.exists()?)
    }
}
