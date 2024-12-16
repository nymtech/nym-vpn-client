// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_compact_ecash::scheme::keygen::KeyPairUser;
use nym_validator_client::{
    nyxd::bip32::DerivationPath, signing::signer::OfflineSigner as _, DirectSecp256k1HdWallet,
};

use crate::{error::Result, jwt::Jwt, VpnApiClientError};

#[derive(Clone, Debug)]
pub struct VpnApiAccount {
    wallet: DirectSecp256k1HdWallet,
}

impl VpnApiAccount {
    #[allow(unused)]
    fn random() -> Self {
        let mnemonic = bip39::Mnemonic::generate(24).unwrap();
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic.clone());
        Self { wallet }
    }

    pub fn id(&self) -> String {
        self.wallet.get_accounts().unwrap()[0].address().to_string()
    }

    pub(crate) fn jwt(&self, vpn_api_unix_epoch: Option<i64>) -> Jwt {
        match vpn_api_unix_epoch {
            Some(epoch) => Jwt::new_secp256k1_synced(&self.wallet, epoch),
            None => Jwt::new_secp256k1(&self.wallet),
        }
    }

    pub fn create_ecash_keypair(&self) -> Result<KeyPairUser> {
        let hd_path = cosmos_derivation_path();
        let extended_private_key = self
            .wallet
            .derive_extended_private_key(&hd_path)
            .map_err(VpnApiClientError::CosmosDeriveFromPath)?;
        Ok(KeyPairUser::new_seeded(
            extended_private_key.private_key().to_bytes(),
        ))
    }
}

impl From<bip39::Mnemonic> for VpnApiAccount {
    fn from(mnemonic: bip39::Mnemonic) -> Self {
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic.clone());
        Self { wallet }
    }
}

fn cosmos_derivation_path() -> DerivationPath {
    nym_config::defaults::COSMOS_DERIVATION_PATH
        .parse()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use crate::types::test_fixtures::{TEST_DEFAULT_MNEMONIC, TEST_DEFAULT_MNEMONIC_ID};

    use super::*;

    #[test]
    fn create_account_from_mnemonic() {
        let account = VpnApiAccount::from(bip39::Mnemonic::parse(TEST_DEFAULT_MNEMONIC).unwrap());
        assert_eq!(account.id(), TEST_DEFAULT_MNEMONIC_ID);
    }
}
