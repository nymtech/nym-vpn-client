use crate::{
    account::{
        StoredAccountMode,
        on_disk::{OnDiskMnemonicStorageError, legacy},
    },
    types::StorableAccount,
};
use bip39::Mnemonic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct StoredAccounts(HashMap<StoredAccountMode, StoredAccount>);

impl StoredAccounts {
    pub(crate) fn insert_account(&mut self, account: StorableAccount) {
        let name = "default".to_string();
        let nonce = 0;
        let stored_account = StoredAccount {
            name,
            mnemonic: account.mnemonic,
            mode: account.mode,
            nonce,
        };
        self.0.insert(account.mode, stored_account);
    }

    pub(crate) fn remove_account(
        &mut self,
        stored_account_mode: Option<StoredAccountMode>,
    ) -> bool {
        if let Some(stored_account_mode) = stored_account_mode {
            self.0.remove(&stored_account_mode).is_some()
        } else {
            let ret = !self.0.is_empty();
            self.0.clear();
            ret
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StoredAccount {
    /// Identifier of the account.
    pub(crate) name: String,

    /// The mnemonic itself.
    pub(crate) mnemonic: Mnemonic,

    /// The mode associated with this account
    pub(crate) mode: StoredAccountMode,

    /// Nonce used to confirm the mnemonic
    pub(crate) nonce: u32,
}

impl TryFrom<StoredAccounts> for Vec<StorableAccount> {
    type Error = OnDiskMnemonicStorageError;

    fn try_from(value: StoredAccounts) -> Result<Self, Self::Error> {
        Ok(value
            .0
            .values()
            .map(|stored| StorableAccount {
                mnemonic: stored.mnemonic.clone(),
                mode: stored.mode,
            })
            .collect())
    }
}

// Kind of a hack here and not how we do it with the versioned daemon config file.
// However I want to preserve the `StoredAccount` type which holds more than a `Vec<StorableAccount>`
// as it will hold stuff we want to use.
impl TryFrom<legacy::StoredAccount> for StoredAccount {
    type Error = OnDiskMnemonicStorageError;

    fn try_from(value: legacy::StoredAccount) -> Result<Self, Self::Error> {
        Ok(StoredAccount {
            name: value.name,
            mnemonic: value.mnemonic,
            mode: value.mode,
            nonce: value.nonce,
        })
    }
}

impl TryFrom<legacy::StoredAccount> for StoredAccounts {
    type Error = OnDiskMnemonicStorageError;

    fn try_from(value: legacy::StoredAccount) -> Result<Self, Self::Error> {
        let stored_account: StoredAccount = value.try_into()?;
        let map = HashMap::from([(stored_account.mode, stored_account); 1]);
        Ok(StoredAccounts(map))
    }
}
