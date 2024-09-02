// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_crypto::asymmetric::ed25519;
use nym_validator_client::{signing::signer::OfflineSigner as _, DirectSecp256k1HdWallet};

use crate::jwt::Jwt;

pub struct Account {
    wallet: DirectSecp256k1HdWallet,
}

impl Account {
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

impl From<bip39::Mnemonic> for Account {
    fn from(mnemonic: bip39::Mnemonic) -> Self {
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic);
        Self { wallet }
    }
}

pub struct Device {
    keypair: Arc<ed25519::KeyPair>,
}

impl Device {
    pub(crate) fn identity_key(&self) -> &ed25519::PublicKey {
        self.keypair.public_key()
    }

    pub(crate) fn jwt(&self) -> Jwt {
        Jwt::new_ecdsa(&self.keypair)
    }
}

impl From<Arc<ed25519::KeyPair>> for Device {
    fn from(keypair: Arc<ed25519::KeyPair>) -> Self {
        Self { keypair }
    }
}

impl From<ed25519::KeyPair> for Device {
    fn from(keypair: ed25519::KeyPair) -> Self {
        Self {
            keypair: Arc::new(keypair),
        }
    }
}
