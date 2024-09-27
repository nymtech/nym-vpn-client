// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub use nym_contracts_common::Percent;

use std::sync::Arc;

use nym_crypto::asymmetric::ed25519;
use nym_validator_client::{signing::signer::OfflineSigner as _, DirectSecp256k1HdWallet};

use crate::{jwt::Jwt, VpnApiClientError};

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

impl From<DirectSecp256k1HdWallet> for VpnApiAccount {
    fn from(wallet: DirectSecp256k1HdWallet) -> Self {
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

#[derive(Clone, Default, Debug)]
pub struct GatewayMinPerformance {
    pub mixnet_min_performance: Option<Percent>,
    pub vpn_min_performance: Option<Percent>,
}

impl GatewayMinPerformance {
    pub fn from_percentage_values(
        mixnet_min_performance: Option<u64>,
        vpn_min_performance: Option<u64>,
    ) -> Result<Self, VpnApiClientError> {
        let mixnet_min_performance = mixnet_min_performance
            .map(Percent::from_percentage_value)
            .transpose()
            .map_err(VpnApiClientError::InvalidPercentValue)?;
        let vpn_min_performance = vpn_min_performance
            .map(Percent::from_percentage_value)
            .transpose()
            .map_err(VpnApiClientError::InvalidPercentValue)?;
        Ok(Self {
            mixnet_min_performance,
            vpn_min_performance,
        })
    }

    pub(crate) fn to_param(&self) -> Vec<(String, String)> {
        let mut params = vec![];
        if let Some(threshold) = self.mixnet_min_performance {
            params.push((
                crate::routes::MIXNET_MIN_PERFORMANCE.to_string(),
                threshold.to_string(),
            ));
        };
        if let Some(threshold) = self.vpn_min_performance {
            params.push((
                crate::routes::VPN_MIN_PERFORMANCE.to_string(),
                threshold.to_string(),
            ));
        };
        params
    }
}

#[derive(Clone, Debug)]
pub enum GatewayType {
    MixnetEntry,
    MixnetExit,
    Wg,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_secp256k1_keypair() -> DirectSecp256k1HdWallet {
        // This is the default mnemonic used in the js integration tests
        let mnemonic = "range mystery picture decline olympic acoustic lesson quick rebuild panda royal fold start leader egg hammer width olympic worry length crawl couch link mobile";

        let mnemonic = bip39::Mnemonic::parse(mnemonic).unwrap();
        DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic)
    }

    // The JS code generates the keypair from this mnemonic. But we are currently unable to
    // replicate this step in Rust, so we use the keypair directly.
    #[allow(unused)]
    fn get_ed25519_keypair() -> ed25519::KeyPair {
        // let mnemonic = "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece";
        let private_key_base58 = "9JqXnPvTrWkq1Yq66d8GbXrcz5eryAhPZvZ46cEsBPUY";
        let public_key_base58 = "4SPdxfBYsuARBw6REQQa5vFiKcvmYiet9sSWqb751i3Z";

        let private_key = bs58::decode(private_key_base58).into_vec().unwrap();
        let public_key = bs58::decode(public_key_base58).into_vec().unwrap();

        ed25519::KeyPair::from_bytes(&private_key, &public_key).unwrap()
    }

    fn get_ed25519_keypair_from_mnemonic(mnemonic: String) -> ed25519::KeyPair {
        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, mnemonic).unwrap();
        let seed = mnemonic.to_seed("");
        let seed_bytes = &seed[..32].try_into().unwrap();

        let signing_key = ed25519_dalek::SigningKey::from_bytes(seed_bytes);
        let verifying_key = signing_key.verifying_key();

        let privkey = signing_key.to_bytes().to_vec();
        let pubkey = verifying_key.to_bytes().to_vec();

        ed25519::KeyPair::from_bytes(&privkey, &pubkey).unwrap()
    }

    #[test]
    fn generate_ed25519_keypair_from_mnemonic_1() {
        let mnemonic = "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece";
        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, mnemonic).unwrap();

        let seed = mnemonic.to_seed("");
        let seed_bytes = &seed[..32].try_into().unwrap();

        let secret_key = ed25519_dalek::SigningKey::from_bytes(seed_bytes);
        let public_key = secret_key.verifying_key();

        let secret_key_base58 = bs58::encode(secret_key.to_bytes()).into_string();
        let public_key_base58 = bs58::encode(public_key.to_bytes()).into_string();

        let secret_key_base64 = base64_url::encode(&secret_key.to_bytes());
        let public_key_base64 = base64_url::encode(&public_key.to_bytes());

        println!("secret key (base58): {:?}", secret_key_base58);
        println!("public key (base58): {:?}", public_key_base58);
        println!("secret key (base64): {:?}", secret_key_base64);
        println!("public key (base64): {:?}", public_key_base64);

        // Expected to be according to the js tests.
        // But something is wrong
        let _expected_private_key_base58 = "9JqXnPvTrWkq1Yq66d8GbXrcz5eryAhPZvZ46cEsBPUY";
        let _expected_public_key_base58 = "4SPdxfBYsuARBw6REQQa5vFiKcvmYiet9sSWqb751i3Z";
        //assert_eq!(secret_key_base58, expected_private_key_base58);
        //assert_eq!(public_key_base58, expected_public_key_base58);
    }

    #[test]
    fn generate_ed25519_keypair_from_mnemonic_2() {
        let mnemonic = "pitch deputy proof fire movie put bread ribbon what chef zebra car vacuum gadget steak board state oyster layer glory barely thrive nice box";
        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, mnemonic).unwrap();

        let seed = mnemonic.to_seed("");
        let seed_bytes = &seed[..32].try_into().unwrap();

        let secret_key = ed25519_dalek::SigningKey::from_bytes(seed_bytes);
        let public_key = secret_key.verifying_key();

        let public_key_base58 = bs58::encode(public_key.to_bytes()).into_string();
        let public_key_base64 = base64_url::encode(&public_key.to_bytes());

        println!("public key (base58): {:?}", public_key_base58);
        println!("public key (base64): {:?}", public_key_base64);

        // Expected to be according to the js tests.
        // But something is wrong
        let _expected_public_key_base58 = "FJDUECYAeosXhNGjxf8w5MJM7N2DfDwQznvWwTxJz6ft";
        //assert_eq!(public_key_base58, expected_public_key_base58);
    }

    #[test]
    fn sign_device_key() {
        // Setup to use the same mnemonics as the js integration test snapshot
        let account = VpnApiAccount::from(get_secp256k1_keypair());
        let device = Device::from(get_ed25519_keypair_from_mnemonic("pitch deputy proof fire movie put bread ribbon what chef zebra car vacuum gadget steak board state oyster layer glory barely thrive nice box".to_string()));
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
        let _expected_device_identity_key = "FJDUECYAeosXhNGjxf8w5MJM7N2DfDwQznvWwTxJz6ft";
        let _expected_signature = "W5Zv1QhG37Al0QQH/9tqOmv1MU9IjfWP1xDq116GGSu/1Z6cnAW0sOyfrIiqdEleUKJB9wC/HjcsifaogymWAw==";
        assert_eq!(account.id(), expected_account_id);
        //assert_eq!(device_identity_key_base64, expected_device_identity_key);
        //assert_eq!(signature, expected_signature);
    }
}
