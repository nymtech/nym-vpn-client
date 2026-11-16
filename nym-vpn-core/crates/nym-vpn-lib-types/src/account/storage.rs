// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::StoredAccountMode;

pub use bip39::Mnemonic;

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StorableAccount {
    pub mnemonic: Mnemonic,
    pub mode: StoredAccountMode,
}

impl StorableAccount {
    pub fn new(mnemonic: bip39::Mnemonic, mode: StoredAccountMode) -> StorableAccount {
        StorableAccount { mnemonic, mode }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl TryFrom<StorableAccount> for nym_vpn_account::VpnAccount {
    type Error = nym_vpn_account::Error;

    fn try_from(account: StorableAccount) -> Result<Self, Self::Error> {
        Self::new(account.mnemonic, account.mode.into())
    }
}

impl std::fmt::Debug for StorableAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorableAccount")
            .field("mnemonic", &"[redacted]")
            .field("mode", &self.mode)
            .finish()
    }
}
