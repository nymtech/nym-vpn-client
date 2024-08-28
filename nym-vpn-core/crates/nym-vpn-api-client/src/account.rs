// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use nym_crypto::asymmetric::ed25519;
use nym_validator_client::{signing::signer::OfflineSigner as _, DirectSecp256k1HdWallet};
use reqwest::{IntoUrl, Url};

use crate::jwt::Jwt;

// const BASE_URL: &str = "https://nymvpn.com/api/public/v1";
const BASE_URL: &str =
    "https://nym-dot-com-git-deploy-canary-nyx-network-staging.vercel.app/api/public/v1";

struct Account {
    wallet: DirectSecp256k1HdWallet,
}

impl Account {
    #[cfg(test)]
    fn random() -> Self {
        let mnemonic = bip39::Mnemonic::generate(24).unwrap();
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic);
        Self { wallet }
    }

    fn id(&self) -> String {
        self.wallet.get_accounts().unwrap()[0].address().to_string()
    }

    fn jwt(&self) -> Jwt {
        Jwt::new_secp256k1(&self.wallet)
    }
}

impl From<bip39::Mnemonic> for Account {
    fn from(mnemonic: bip39::Mnemonic) -> Self {
        let wallet = DirectSecp256k1HdWallet::from_mnemonic("n", mnemonic);
        Self { wallet }
    }
}

struct Device {
    keypair: ed25519::KeyPair,
}

impl Device {
    fn jwt(&self) -> Jwt {
        Jwt::new_ecdsa(&self.keypair)
    }
}

impl From<ed25519::KeyPair> for Device {
    fn from(keypair: ed25519::KeyPair) -> Self {
        Self { keypair }
    }
}

mod routes {
    pub(super) const ACCOUNT: &str = "account";
}

pub struct AccountClient {
    nym_vpn_api_url: Url,
}

impl AccountClient {
    pub fn new(nym_vpn_api_url: Url) -> Self {
        Self { nym_vpn_api_url }
    }

    fn path(&self, parts: &[&str]) -> String {
        let mut path = vec![self.nym_vpn_api_url.as_str()];
        path.extend(parts);
        path.join("/")
    }

    fn get<U: IntoUrl>(
        &self,
        url: U,
        account: &Account,
        device: &Device,
    ) -> reqwest::RequestBuilder {
        let reqwest_builder = reqwest::Client::new().get(url);
        let reqwest_builder =
            crate::headers::add_account_auth_header(reqwest_builder, account.jwt().to_string());
        let reqwest_builder =
            crate::headers::add_device_auth_header(reqwest_builder, device.jwt().to_string());
        reqwest_builder
    }

    pub async fn get_account(&self, account: &Account, device: &Device) {
        dbg!(&account.jwt());
        dbg!(&device.jwt());
        let url = self.path(&[routes::ACCOUNT, &account.id()]);
        let response = self.get(url, account, device).send().await.unwrap();
        dbg!(&response);
        let text = response.text().await.unwrap();
        dbg!(&text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_mnemonic() -> bip39::Mnemonic {
        let mnemonic = "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece";
        bip39::Mnemonic::parse(mnemonic).unwrap()
    }

    fn get_ed25519_keypair() -> ed25519::KeyPair {
        // let mnemonic = "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece";
        let private_key_base58 = "9JqXnPvTrWkq1Yq66d8GbXrcz5eryAhPZvZ46cEsBPUY";
        let public_key_base58 = "4SPdxfBYsuARBw6REQQa5vFiKcvmYiet9sSWqb751i3Z";

        let private_key = bs58::decode(private_key_base58).into_vec().unwrap();
        let public_key = bs58::decode(public_key_base58).into_vec().unwrap();

        ed25519::KeyPair::from_bytes(&private_key, &public_key).unwrap()
    }

    #[tokio::test]
    async fn get_account() {
        let account = Account::from(get_mnemonic());
        let device = Device::from(get_ed25519_keypair());
        let client = AccountClient::new(BASE_URL.parse().unwrap());
        client.get_account(&account, &device).await;
    }
}
