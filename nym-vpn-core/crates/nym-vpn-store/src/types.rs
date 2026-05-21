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
    #[serde(default)]
    pub is_locally_generated: bool,
    #[serde(default)]
    pub is_registered_with_api: bool,
    #[serde(default)]
    pub is_backup_confirmed: bool,
}

impl StorableAccount {
    pub fn new(mnemonic: bip39::Mnemonic, mode: StoredAccountMode) -> StorableAccount {
        StorableAccount {
            mnemonic,
            mode,
            is_locally_generated: false,
            is_registered_with_api: false,
            is_backup_confirmed: false,
        }
    }

    pub fn new_locally_generated(
        mnemonic: bip39::Mnemonic,
        mode: StoredAccountMode,
    ) -> StorableAccount {
        StorableAccount {
            mnemonic,
            mode,
            is_locally_generated: true,
            is_registered_with_api: false,
            is_backup_confirmed: false,
        }
    }
}

impl std::fmt::Debug for StorableAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorableAccount")
            .field("mnemonic", &"[redacted]")
            .field("mode", &self.mode)
            .field("is_locally_generated", &self.is_locally_generated)
            .field("is_registered_with_api", &self.is_registered_with_api)
            .field("is_backup_confirmed", &self.is_backup_confirmed)
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

#[cfg(test)]
mod flag_tests {
    use super::*;

    fn mnemonic() -> bip39::Mnemonic {
        "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece".parse().unwrap()
    }

    #[test]
    fn new_storable_account_defaults_flags_to_false() {
        let a = StorableAccount::new(mnemonic(), StoredAccountMode::Api);
        assert!(!a.is_locally_generated);
        assert!(!a.is_registered_with_api);
        assert!(!a.is_backup_confirmed);
    }

    #[test]
    fn locally_generated_constructor_sets_flag() {
        let a = StorableAccount::new_locally_generated(mnemonic(), StoredAccountMode::Api);
        assert!(a.is_locally_generated);
        assert!(!a.is_registered_with_api);
        assert!(!a.is_backup_confirmed);
    }
}
