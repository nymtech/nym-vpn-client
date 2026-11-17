// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_secure_enclave::{KeychainError, KeychainItem};
use zeroize::Zeroizing;

use super::{AccountInformationStorage, StoredAccountInformation};

#[derive(Debug, thiserror::Error)]
pub enum KeychainAccountStorageError {
    #[error("account information keychain error")]
    Keychain(#[from] KeychainError),

    #[error("failed to serialize account information")]
    Serialize(#[source] serde_json::Error),

    #[error("failed to deserialize account information")]
    Deserialize(#[source] serde_json::Error),
}

pub type Result<T, E = KeychainAccountStorageError> = std::result::Result<T, E>;

/// Stores the account information in the keychain.
pub struct KeychainAccountStorage {
    item: KeychainItem,
}

impl KeychainAccountStorage {
    pub fn new(item: KeychainItem) -> Self {
        Self { item }
    }
}

#[async_trait::async_trait]
impl AccountInformationStorage for KeychainAccountStorage {
    type StorageError = KeychainAccountStorageError;

    async fn load_account(&self) -> Result<Option<StoredAccountInformation>> {
        self.item
            .get_value()
            .await?
            .map(|value| {
                serde_json::from_slice::<StoredAccountInformation>(&value)
                    .map_err(KeychainAccountStorageError::Deserialize)
            })
            .transpose()
    }

    async fn store_account(&self, state: StoredAccountInformation) -> Result<()> {
        let value = serde_json::to_vec(&state)
            .map(Zeroizing::new)
            .map_err(KeychainAccountStorageError::Serialize)?;
        Ok(self.item.set_value(&value).await?)
    }

    async fn remove_account(&self) -> Result<()> {
        Ok(self.item.delete()?)
    }
}
