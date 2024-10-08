// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_validator_client::{signing::signer::OfflineSigner as _, DirectSecp256k1HdWallet};

use crate::jwt::Jwt;

use super::Device;

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

    // Base64 encoded signature
    pub(crate) fn sign_device_key(&self, device: &Device) -> String {
        let accounts = self.wallet.get_accounts().unwrap();
        let address = accounts[0].address();

        let device_identity_key = device.identity_key().to_bytes();

        let signature = self
            .wallet
            .sign_raw(address, device_identity_key)
            .unwrap()
            .to_bytes()
            .to_vec();
        base64_url::encode(&signature)
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
    use crate::types::test_fixtures::{
        DEFAULT_DEVICE_MNEMONIC, DEFAULT_MNEMONIC, DEFAULT_MNEMONIC_ID,
    };

    use super::*;

    #[test]
    fn create_account_from_mnemonic() {
        let account = VpnApiAccount::from(bip39::Mnemonic::parse(DEFAULT_MNEMONIC).unwrap());
        assert_eq!(account.id(), DEFAULT_MNEMONIC_ID);
    }

    #[test]
    fn sign_device_key() {
        // Setup to use the same mnemonics as the js integration test snapshot
        let account = VpnApiAccount::from(bip39::Mnemonic::parse(DEFAULT_MNEMONIC).unwrap());
        let device = Device::from(bip39::Mnemonic::parse(DEFAULT_DEVICE_MNEMONIC).unwrap());
        println!("account id: {}", account.id());
        println!(
            "device identity key (base58): {:?}",
            device.identity_key().to_base58_string()
        );
        let device_identity_key_bytes = device.identity_key().to_bytes();
        let device_identity_key_base64 = base64_url::encode(&device_identity_key_bytes);
        println!(
            "device identity key (base64): {:?}",
            device_identity_key_base64,
        );
        let signature = account.sign_device_key(&device);
        println!("signature: {signature}");

        // From the js integration tests
        let expected_account_id = "n1sslaag27wfydyrvyua72hg5e0vteglxrs8nw3c";
        let expected_device_identity_key = "FJDUECYAeosXhNGjxf8w5MJM7N2DfDwQznvWwTxJz6ft";
        let expected_signature = "W5Zv1QhG37Al0QQH/9tqOmv1MU9IjfWP1xDq116GGSu/1Z6cnAW0sOyfrIiqdEleUKJB9wC/HjcsifaogymWAw==";
        assert_eq!(account.id(), expected_account_id);
        assert_eq!(device.identity_key().to_string(), expected_device_identity_key);
        assert_eq!(signature, expected_signature);
    }
}
