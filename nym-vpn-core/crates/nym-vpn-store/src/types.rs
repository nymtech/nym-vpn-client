// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RawWireguardKeys {
    pub gateway_id_bs58: String,
    pub entry_private_key_bs58: String,
    pub exit_private_key_bs58: String,
    pub expiration_time: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct StorableAccount {
    pub mnemonic: bip39::Mnemonic,
    pub mode: StoredAccountMode,
}

impl StorableAccount {
    pub fn new(mnemonic: bip39::Mnemonic, mode: StoredAccountMode) -> StorableAccount {
        StorableAccount { mnemonic, mode }
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

/// Defines the mode of operation of the associated account.
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StoredAccountMode {
    /// Account works in the API mode, i.e. the subscription is managed
    /// by the VPN API which provides required ticketbooks
    #[default]
    Api,

    /// Account works in the decentralised mode, i.e. there is no associated subscription
    /// and the account uses its own funds for obtaining required ticketbooks
    Decentralised,

    /// Account works in the API mode, but the mnemonic is derived from the Privy
    /// wallet private key.
    Privy,
}

#[cfg(test)]
mod tests {
    use zeroize::ZeroizeOnDrop;

    fn _assert_zeroize_on_drop<T: ZeroizeOnDrop>() {}

    #[test]
    fn mnemonic_zeroize_feature_is_activated() {
        _assert_zeroize_on_drop::<bip39::Mnemonic>();
    }
}
