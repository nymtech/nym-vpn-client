// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_crypto::asymmetric::ed25519;
use nym_validator_client::DirectSecp256k1HdWallet;

struct AccountClient {
    client: reqwest::Client,
}

impl AccountClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_account(&self) {
        // Create reqwest client, and setup jwt headers.
        let base_url =
            "https://nym-dot-com-git-deploy-canary-nyx-network-staging.vercel.app/api/public/v1";
        let url = format!("{}/account/{}", base_url, 123);
        let reqwest_builder = self.client.get(url);

        let wallet = get_secp256k1_keypair();
        let device_jwt = crate::jwt::Jwt::new_secp256k1(&wallet);
        dbg!(&device_jwt);
        dbg!(&device_jwt.jwt());

        let device_auth_header = device_jwt.jwt().to_string();

        let reqwest_builder =
            crate::headers::add_device_auth_header(reqwest_builder, device_auth_header);

        let key_pair = get_ed25519_keypair();
        let account_jwt = crate::jwt::Jwt::new_ecdsa(&key_pair);

        let account_auth_header = account_jwt.jwt().to_string();

        let reqwest_builder =
            crate::headers::add_account_auth_header(reqwest_builder, account_auth_header);

        let response = reqwest_builder.send().await.unwrap();
        dbg!(&response);
        let text = response.text().await.unwrap();
        dbg!(&text);
    }
}

fn get_secp256k1_keypair() -> DirectSecp256k1HdWallet {
    let mnemonic = "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece";
    let mnemonic = bip39::Mnemonic::parse(mnemonic).unwrap();
    DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic)
}

fn get_ed25519_keypair() -> ed25519::KeyPair {
    // let mnemonic = "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece";
    let private_key_base58 = "9JqXnPvTrWkq1Yq66d8GbXrcz5eryAhPZvZ46cEsBPUY";
    let public_key_base58 = "4SPdxfBYsuARBw6REQQa5vFiKcvmYiet9sSWqb751i3Z";

    let private_key = bs58::decode(private_key_base58).into_vec().unwrap();
    let public_key = bs58::decode(public_key_base58).into_vec().unwrap();

    ed25519::KeyPair::from_bytes(&private_key, &public_key).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_account() {
        let client = AccountClient::new();
        client.get_account().await;
    }
}
