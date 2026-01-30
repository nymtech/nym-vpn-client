// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::AccountInformationStorage;
use crate::types::{StorableAccount, StoredAccountMode};
use bip39::Mnemonic;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Deserialize)]
struct StoredAccount {
    /// The mnemonic itself.
    mnemonic: Mnemonic,

    /// The mode associated with this account
    mode: StoredAccountMode,
}

#[derive(Default)]
pub struct InMemoryAccountStorage {
    accounts: Mutex<HashMap<StoredAccountMode, StoredAccount>>,
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

    async fn load_accounts(&self) -> Result<Vec<StorableAccount>, InMemoryAccountStorageError> {
        let guard = self.accounts.lock().await;
        Ok(guard
            .values()
            .map(|stored| StorableAccount {
                mnemonic: stored.mnemonic.clone(),
                mode: stored.mode,
            })
            .collect())
    }

    async fn store_account(
        &self,
        account: StorableAccount,
    ) -> Result<(), InMemoryAccountStorageError> {
        let stored_account = StoredAccount {
            mnemonic: account.mnemonic,
            mode: account.mode,
        };

        let mut guard = self.accounts.lock().await;
        guard.insert(stored_account.mode, stored_account);
        Ok(())
    }

    async fn remove_account(
        &self,
        stored_account_mode: Option<StoredAccountMode>,
    ) -> Result<(), InMemoryAccountStorageError> {
        let mut guard = self.accounts.lock().await;

        if let Some(stored_account_mode) = stored_account_mode {
            if guard.contains_key(&stored_account_mode) {
                guard.remove(&stored_account_mode);
                Ok(())
            } else {
                Err(InMemoryAccountStorageError::NoMnemonicStored)
            }
        } else {
            guard.clear();
            Ok(())
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

        let loaded_accounts = storage.load_accounts().await.unwrap();
        assert_eq!(loaded_accounts, vec![account]);
    }

    #[tokio::test]
    async fn load_non_existing_mnemonic_returns_empty_vec() {
        let storage = InMemoryAccountStorage::default();

        let loaded_accounts = storage.load_accounts().await.unwrap();
        assert!(loaded_accounts.is_empty());
    }
}
