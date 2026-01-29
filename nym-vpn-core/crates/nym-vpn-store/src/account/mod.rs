// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

pub mod ephemeral;
pub mod on_disk;

pub use crate::types::{StorableAccount, StoredAccountMode};
pub use bip39::Mnemonic;

#[async_trait::async_trait]
pub trait AccountInformationStorage {
    type StorageError: Error + Send + Sync + 'static;

    async fn load_accounts(&self) -> Result<Vec<StorableAccount>, Self::StorageError>;

    async fn store_account(&self, account: StorableAccount) -> Result<(), Self::StorageError>;

    /// If `stored_account_mode` is `None` then remove all the accounts
    async fn remove_account(
        &self,
        stored_account_mode: Option<StoredAccountMode>,
    ) -> Result<(), Self::StorageError>;

    async fn is_account_stored(
        &self,
        stored_account_mode: StoredAccountMode,
    ) -> Result<bool, Self::StorageError> {
        let accounts = self.load_accounts().await?;
        Ok(accounts
            .iter()
            .any(|account| account.mode == stored_account_mode))
    }
}

// There is some duplication here as `StoredAccount` already contains the `StoredAccountMode`,
// however this does enforce storage of only one account of each type.
#[derive(Default, Serialize, Deserialize)]
struct StoredAccounts(HashMap<StoredAccountMode, StoredAccount>);

impl StoredAccounts {
    fn with_account(account: StoredAccount) -> Self {
        let mut accounts = HashMap::new();
        accounts.insert(account.mode, account);
        StoredAccounts(accounts)
    }
}

#[derive(Serialize, Deserialize)]
struct StoredAccount {
    /// Identifier of the account.
    name: String,

    /// The mnemonic itself.
    mnemonic: Mnemonic,

    /// The mode associated with this account
    /// note that it won't exist for legacy data
    #[serde(default)]
    mode: StoredAccountMode,

    /// Nonce used to confirm the mnemonic
    nonce: Nonce,
}

impl std::fmt::Debug for StoredAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredAccount")
            .field("name", &self.name)
            .field("mnemonic", &"[redacted]")
            .field("mode", &self.mode)
            .field("nonce", &self.nonce)
            .finish()
    }
}

impl From<StoredAccount> for StorableAccount {
    fn from(account: StoredAccount) -> Self {
        StorableAccount {
            mnemonic: account.mnemonic,
            mode: account.mode,
        }
    }
}

type Nonce = u32;

#[cfg(test)]
pub(crate) mod test_fixtures {
    use super::*;

    pub(crate) fn mnemonic_fixture() -> bip39::Mnemonic {
        "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece".parse().unwrap()
    }

    pub(crate) fn account_fixture() -> StorableAccount {
        StorableAccount {
            mnemonic: mnemonic_fixture(),
            mode: StoredAccountMode::Api,
        }
    }
}
