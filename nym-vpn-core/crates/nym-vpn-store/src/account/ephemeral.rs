// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AccountInformationStorage, StoredAccount};
use crate::types::StorableAccount;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct InMemoryAccountStorage {
    account: Mutex<Option<StoredAccount>>,
}

#[derive(Debug, thiserror::Error)]
pub enum InMemoryAccountStorageError {
    #[error("no passphrase stored")]
    NoMnemonicStored,

    #[error("passphrase already stored")]
    MnemonicAlreadyStored,
}

#[async_trait::async_trait]
impl AccountInformationStorage for InMemoryAccountStorage {
    type StorageError = InMemoryAccountStorageError;

    async fn store_account(
        &self,
        account: StorableAccount,
    ) -> Result<(), InMemoryAccountStorageError> {
        let name = "default".to_string();
        let nonce = 0;
        let stored_account = StoredAccount {
            name,
            mnemonic: account.mnemonic,
            mode: account.mode,
            nonce,
        };
        let mut handle = self.account.lock().await;

        // Store the mnemonic if it's currently None
        if handle.is_none() {
            *handle = Some(stored_account);
            Ok(())
        } else {
            Err(InMemoryAccountStorageError::MnemonicAlreadyStored)
        }
    }

    async fn load_account(&self) -> Result<Option<StorableAccount>, InMemoryAccountStorageError> {
        Ok(self
            .account
            .lock()
            .await
            .as_ref()
            .map(|stored| StorableAccount {
                mnemonic: stored.mnemonic.clone(),
                mode: stored.mode,
            }))
    }

    async fn remove_account(&self) -> Result<(), InMemoryAccountStorageError> {
        let mut handle = self.account.lock().await;

        if handle.is_some() {
            *handle = None;
            Ok(())
        } else {
            Err(InMemoryAccountStorageError::NoMnemonicStored)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::test_fixtures::account_fixture;

    #[tokio::test]
    async fn store_and_load_account() {
        let account = account_fixture();

        let storage = InMemoryAccountStorage::default();
        storage.store_account(account.clone()).await.unwrap();

        let loaded_mnemonic = storage.load_account().await.unwrap();
        assert_eq!(loaded_mnemonic, Some(account));
    }

    #[tokio::test]
    async fn load_non_existing_mnemonic_returns_none() {
        let storage = InMemoryAccountStorage::default();

        let loaded_mnemonic = storage.load_account().await.unwrap();
        assert_eq!(loaded_mnemonic, None);
    }
}
