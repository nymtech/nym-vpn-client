// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_validator_client::{signing::signer::OfflineSigner as _, DirectSecp256k1HdWallet};

use crate::jwt::Jwt;

#[derive(Clone, Debug)]
pub struct VpnApiAccount {
    wallet: DirectSecp256k1HdWallet,
}

impl VpnApiAccount {
    #[allow(unused)]
    fn random() -> Self {
        let mnemonic = bip39::Mnemonic::generate(24).unwrap();
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic);
        Self { wallet }
    }

    pub fn id(&self) -> String {
        self.wallet.get_accounts().unwrap()[0].address().to_string()
    }

    pub(crate) fn jwt(&self) -> Jwt {
        Jwt::new_secp256k1(&self.wallet)
    }
}

impl From<bip39::Mnemonic> for VpnApiAccount {
    fn from(mnemonic: bip39::Mnemonic) -> Self {
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic);
        Self { wallet }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::test_fixtures::{DEFAULT_MNEMONIC, DEFAULT_MNEMONIC_ID};

    use super::*;

    #[test]
    fn create_account_from_mnemonic() {
        let account = VpnApiAccount::from(bip39::Mnemonic::parse(DEFAULT_MNEMONIC).unwrap());
        assert_eq!(account.id(), DEFAULT_MNEMONIC_ID);
    }
}
