use crate::{account::StoredAccountMode, types::StorableAccount};
use bip39::Mnemonic;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct StoredAccount {
    /// Identifier of the account.
    pub(crate) name: String,

    /// The mnemonic itself.
    pub(crate) mnemonic: Mnemonic,

    /// The mode associated with this account
    /// note that it won't exist for legacy data
    /// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// This comment implies there are at least two versions of the legacy mnemonic.json file
    #[serde(default)]
    pub(crate) mode: StoredAccountMode,

    /// Nonce used to confirm the mnemonic
    pub(crate) nonce: u32,
}

impl From<StoredAccount> for Vec<StorableAccount> {
    fn from(account: StoredAccount) -> Self {
        vec![StorableAccount {
            mnemonic: account.mnemonic,
            mode: account.mode,
        }]
    }
}
