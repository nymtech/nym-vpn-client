// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::error::Error;

pub mod ephemeral;
pub mod on_disk;

pub use crate::types::{StorableAccount, StoredAccountMode};
pub use bip39::Mnemonic;

#[async_trait::async_trait]
pub trait AccountInformationStorage {
    type StorageError: Error + Send + Sync + 'static;

    async fn load_account(&self) -> Result<Option<StorableAccount>, Self::StorageError>; // None means no error, but no mnemonic stored
    async fn store_account(&self, account: StorableAccount) -> Result<(), Self::StorageError>;
    async fn remove_account(&self) -> Result<(), Self::StorageError>;
    async fn is_account_stored(&self) -> Result<bool, Self::StorageError> {
        self.load_account()
            .await
            .map(|maybe_account| maybe_account.is_some())
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

    /// True when the mnemonic was generated locally via CreateAccount (vs imported).
    #[serde(default)]
    is_locally_generated: bool,

    /// True after a successful registration with nym-vpn-api.
    #[serde(default)]
    is_registered_with_api: bool,

    /// True after the user has confirmed they saved the recovery phrase.
    #[serde(default)]
    is_backup_confirmed: bool,
}

impl std::fmt::Debug for StoredAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredAccount")
            .field("name", &self.name)
            .field("mnemonic", &"[redacted]")
            .field("mode", &self.mode)
            .field("nonce", &self.nonce)
            .field("is_locally_generated", &self.is_locally_generated)
            .field("is_registered_with_api", &self.is_registered_with_api)
            .field("is_backup_confirmed", &self.is_backup_confirmed)
            .finish()
    }
}

impl From<StoredAccount> for StorableAccount {
    fn from(account: StoredAccount) -> Self {
        StorableAccount {
            mnemonic: account.mnemonic,
            mode: account.mode,
            is_locally_generated: account.is_locally_generated,
            is_registered_with_api: account.is_registered_with_api,
            is_backup_confirmed: account.is_backup_confirmed,
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
            is_locally_generated: false,
            is_registered_with_api: false,
            is_backup_confirmed: false,
        }
    }
}

#[cfg(test)]
mod stored_account_tests {
    use super::*;

    #[test]
    fn stored_account_round_trips_new_flags() {
        let mnemonic = test_fixtures::mnemonic_fixture();
        let stored = StoredAccount {
            name: "default".to_string(),
            mnemonic: mnemonic.clone(),
            mode: StoredAccountMode::Api,
            nonce: 0,
            is_locally_generated: true,
            is_registered_with_api: true,
            is_backup_confirmed: false,
        };
        let storable: StorableAccount = stored.into();
        assert!(storable.is_locally_generated);
        assert!(storable.is_registered_with_api);
        assert!(!storable.is_backup_confirmed);
    }
}
