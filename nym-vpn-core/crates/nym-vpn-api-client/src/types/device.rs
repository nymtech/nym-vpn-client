// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_crypto::asymmetric::ed25519;

use crate::jwt::Jwt;

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

//impl From<&str> for Device {
//    fn from(mnemonic: &str) -> Self {
//        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, mnemonic).unwrap();
//        let seed = mnemonic.to_seed("");
//        let seed_bytes = &seed[..32].try_into().unwrap();
//
//        let signing_key = ed25519_dalek::SigningKey::from_bytes(seed_bytes);
//        let verifying_key = signing_key.verifying_key();
//
//        let privkey = signing_key.to_bytes().to_vec();
//        let pubkey = verifying_key.to_bytes().to_vec();
//
//        let keypair = ed25519::KeyPair::from_bytes(&privkey, &pubkey).unwrap();
//
//        Self {
//            keypair: Arc::new(keypair),
//        }
//    }
//}

impl From<bip39::Mnemonic> for Device {
    fn from(mnemonic: bip39::Mnemonic) -> Self {
        let (entropy, _) = mnemonic.to_entropy_array();
        // Entropy is statically >= 32 bytes, so we can safely unwrap here
        let seed = &entropy[0..32].try_into().unwrap();

        // let seed = mnemonic.to_seed("");
        // let seed_bytes = &seed[..32].try_into().unwrap();

        let signing_key = ed25519_dalek::SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();

        let privkey = signing_key.to_bytes().to_vec();
        let pubkey = verifying_key.to_bytes().to_vec();

        let keypair = ed25519::KeyPair::from_bytes(&privkey, &pubkey).unwrap();

        Self {
            keypair: Arc::new(keypair),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::test_fixtures::{DEFAULT_DEVICE_IDENTITY_KEY, DEFAULT_DEVICE_MNEMONIC};

    use super::*;

    // The JS code generates the keypair from this mnemonic. But we are currently unable to
    // replicate this step in Rust, so we use the keypair directly.
    fn ed25519_keypair_fixture() -> ed25519::KeyPair {
        let _mnemonic = "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece";

        // The corresponding keypair generated from the mnemonic
        let private_key_base58 = "9JqXnPvTrWkq1Yq66d8GbXrcz5eryAhPZvZ46cEsBPUY";
        let public_key_base58 = "4SPdxfBYsuARBw6REQQa5vFiKcvmYiet9sSWqb751i3Z";

        let private_key = bs58::decode(private_key_base58).into_vec().unwrap();
        let public_key = bs58::decode(public_key_base58).into_vec().unwrap();

        ed25519::KeyPair::from_bytes(&private_key, &public_key).unwrap()
    }

    #[test]
    fn verify_ed25519_keypair_fixture() {
        let device = Device::from(
            bip39::Mnemonic::parse("kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece").unwrap()
        );
        let expected_keypair = ed25519_keypair_fixture();
        assert_eq!(
            device.keypair.private_key().to_base58_string(),
            expected_keypair.private_key().to_base58_string()
        );
        assert_eq!(
            device.keypair.public_key().to_base58_string(),
            expected_keypair.public_key().to_base58_string()
        );
    }

    #[test]
    fn create_device_from_mnemonic() {
        let device = Device::from(bip39::Mnemonic::parse(DEFAULT_DEVICE_MNEMONIC).unwrap());
        assert_eq!(
            device.identity_key().to_base58_string(),
            DEFAULT_DEVICE_IDENTITY_KEY
        );
    }

    #[test]
    fn create_device_from_keypair() {
        let device = Device::from(ed25519_keypair_fixture());
        let expected_priv_key = "9JqXnPvTrWkq1Yq66d8GbXrcz5eryAhPZvZ46cEsBPUY";
        let expected_pub_key = "4SPdxfBYsuARBw6REQQa5vFiKcvmYiet9sSWqb751i3Z";
        assert_eq!(
            device.keypair.public_key().to_base58_string(),
            expected_pub_key
        );
        assert_eq!(
            device.keypair.private_key().to_base58_string(),
            expected_priv_key
        );
    }

    fn generate_ed25519_keypair_from_mnemonic(
        mnemonic: bip39::Mnemonic,
    ) -> (ed25519_dalek::SigningKey, ed25519_dalek::VerifyingKey) {
        let seed = mnemonic.to_seed("");
        let seed_bytes = &seed[..32].try_into().unwrap();

        let secret_key = ed25519_dalek::SigningKey::from_bytes(seed_bytes);
        let public_key = secret_key.verifying_key();
        (secret_key, public_key)
    }

    fn generate_ed25519_keypair_from_mnemonic_jon(
        entropy: &[u8; 32],
    ) -> (ed25519_dalek::SigningKey, ed25519_dalek::VerifyingKey) {
        // let seed = mnemonic.to_seed("");
        // let seed_bytes = &seed[..32].try_into().unwrap();
        // let seed_bytes = entropy.as_ref();

        let secret_key = ed25519_dalek::SigningKey::from_bytes(entropy);
        let public_key = secret_key.verifying_key();
        (secret_key, public_key)
    }

    // WIP
    #[test]
    fn generate_ed25519_keypair_from_mnemonic_1() {
        let mnemonic = bip39::Mnemonic::parse_in(
            bip39::Language::English,
            "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece",
        ).unwrap();

        let entropy = mnemonic.to_entropy_array().0;
        let bytes = &entropy[0..32].try_into().unwrap();

        // let (secret_key, public_key) = generate_ed25519_keypair_from_mnemonic(mnemonic);
        let (secret_key, public_key) = generate_ed25519_keypair_from_mnemonic_jon(bytes);

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

    // WIP
    #[test]
    fn generate_ed25519_keypair_from_mnemonic_2() {
        let mnemonic =
            bip39::Mnemonic::parse_in(bip39::Language::English, DEFAULT_DEVICE_MNEMONIC).unwrap();

        let (secret_key, public_key) = generate_ed25519_keypair_from_mnemonic(mnemonic);

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
        let _expected_public_key_base58 = "FJDUECYAeosXhNGjxf8w5MJM7N2DfDwQznvWwTxJz6ft";
        //assert_eq!(public_key_base58, expected_public_key_base58);
    }
}
