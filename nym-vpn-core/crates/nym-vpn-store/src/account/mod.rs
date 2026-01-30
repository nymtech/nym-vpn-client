// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod ephemeral;
pub mod on_disk;

use std::error::Error;

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
