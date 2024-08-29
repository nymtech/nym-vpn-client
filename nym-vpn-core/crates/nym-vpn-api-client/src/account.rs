// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_crypto::asymmetric::ed25519;
use nym_validator_client::{signing::signer::OfflineSigner as _, DirectSecp256k1HdWallet};
use reqwest::{IntoUrl, StatusCode, Url};
use serde::{Deserialize, Serialize};

use crate::{headers::DEVICE_AUTHORIZATION_HEADER, jwt::Jwt};

// const BASE_URL: &str = "https://nymvpn.com/api/public";
const BASE_URL: &str =
    "https://nym-dot-com-git-deploy-canary-nyx-network-staging.vercel.app/api/public";

pub struct Account {
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

pub struct Device {
    keypair: Arc<ed25519::KeyPair>,
}

impl Device {
    fn id(&self) -> String {
        self.keypair.public_key().to_base58_string()
    }

    fn jwt(&self) -> Jwt {
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

mod routes {
    pub const V1: &str = "v1";
    pub const ACCOUNT: &str = "account";
    pub const SUMMARY: &str = "summary";
    pub const DEVICE: &str = "device";
    pub const ACTIVE: &str = "active";
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterDeviceRequest {
    device_identity_key: String,
    signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountResponse {
    created_on_utc: String,
    last_updated_utc: String,
    account_addr: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("unknown")]
    Unknown,

    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("forbidden: {0}")]
    Forbidden(serde_json::Value),
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

    fn get_with_account<U: IntoUrl>(&self, url: U, account: &Account) -> reqwest::RequestBuilder {
        let builder = reqwest::Client::new().get(url);
        crate::headers::add_account_auth_header(builder, account.jwt().to_string())
    }

    fn get_with_device<U: IntoUrl>(
        &self,
        url: U,
        account: &Account,
        device: &Device,
    ) -> reqwest::RequestBuilder {
        let builder = reqwest::Client::new().get(url);
        let builder = crate::headers::add_account_auth_header(builder, account.jwt().to_string());
        crate::headers::add_device_auth_header(builder, device.jwt().to_string())
    }

    fn post_with_account<U: IntoUrl>(&self, url: U, account: &Account) -> reqwest::RequestBuilder {
        let builder = reqwest::Client::new().post(url);
        crate::headers::add_account_auth_header(builder, account.jwt().to_string())
    }

    fn post_with_device<U: IntoUrl>(
        &self,
        url: U,
        account: &Account,
        device: &Device,
    ) -> reqwest::RequestBuilder {
        let builder = reqwest::Client::new().post(url);
        let builder = crate::headers::add_account_auth_header(builder, account.jwt().to_string());
        crate::headers::add_device_auth_header(builder, device.jwt().to_string())
    }

    fn delete_with_account<U: IntoUrl>(
        &self,
        url: U,
        account: &Account,
    ) -> reqwest::RequestBuilder {
        let builder = reqwest::Client::new().delete(url);
        crate::headers::add_account_auth_header(builder, account.jwt().to_string())
    }

    fn delete_with_device<U: IntoUrl>(
        &self,
        url: U,
        account: &Account,
        device: &Device,
    ) -> reqwest::RequestBuilder {
        let builder = reqwest::Client::new().delete(url);
        let builder = crate::headers::add_account_auth_header(builder, account.jwt().to_string());
        crate::headers::add_device_auth_header(builder, device.jwt().to_string())
    }

    pub async fn get_account(&self, account: &Account) -> Result<NymVpnAccountResponse, ApiError> {
        dbg!(&account.jwt());
        let url = self.path(&[routes::V1, routes::ACCOUNT, &account.id()]);
        let response = self.get_with_account(url, account).send().await?;
        dbg!(&response);

        // let text = response.text().await.unwrap();
        // dbg!(&text);

        let json = match response.status() {
            StatusCode::OK => response.json().await.map_err(ApiError::from),
            StatusCode::FORBIDDEN => Err(ApiError::Forbidden(response.json().await.unwrap())),
            _ => Err(ApiError::Unknown),
        };
        dbg!(&json);
        json
    }

    pub async fn get_account_summary(&self, account: &Account) {
        let url = self.path(&[routes::V1, routes::ACCOUNT, &account.id(), routes::SUMMARY]);
        let response = self.get_with_account(url, account).send().await.unwrap();
        dbg!(&response);
        let text = response.text().await.unwrap();
        dbg!(&text);
    }

    pub async fn remove_account(&self, account: &Account) {
        let url = self.path(&[routes::V1, routes::ACCOUNT, &account.id()]);
        let response = self.delete_with_account(url, account).send().await.unwrap();
        dbg!(&response);
        let text = response.text().await.unwrap();
        dbg!(&text);
    }

    pub async fn get_devices(&self, account: &Account) {
        let url = self.path(&[routes::V1, routes::ACCOUNT, &account.id(), routes::DEVICE]);
        let response = self.get_with_account(url, account).send().await.unwrap();
        dbg!(&response);
        let text = response.text().await.unwrap();
        dbg!(&text);
    }

    pub async fn register_device(&self, account: &Account, device: &Device) {
        let url = self.path(&[routes::V1, routes::ACCOUNT, &account.id(), routes::DEVICE]);
        let request = RegisterDeviceRequest {
            device_identity_key: device.keypair.public_key().to_base58_string(),
            signature: device.jwt().to_string(),
        };
        let response = self
            .post_with_device(url, account, device)
            .json(&request)
            .send()
            .await
            .unwrap();
        dbg!(&response);
        let text = response.text().await.unwrap();
        dbg!(&text);
    }

    pub async fn get_active_devices(&self, account: &Account) {
        let url = self.path(&[
            routes::V1,
            routes::ACCOUNT,
            &account.id(),
            routes::DEVICE,
            routes::ACTIVE,
        ]);
        let response = self.get_with_account(url, account).send().await.unwrap();
        dbg!(&response);
        let text = response.text().await.unwrap();
        dbg!(&text);
    }

    pub async fn get_device_by_id(&self, account: &Account, device: &Device) {
        let url = self.path(&[
            routes::V1,
            routes::ACCOUNT,
            &account.id(),
            routes::DEVICE,
            &device.id(),
        ]);
        let response = self.get_with_account(url, account).send().await.unwrap();
        dbg!(&response);
        let text = response.text().await.unwrap();
        dbg!(&text);
    }
}

// #[allow(async_fn_in_trait)]
// pub trait VpnApiClientExt2: nym_http_api_client::ApiClient {
//     fn get_with_account<U: IntoUrl>(&self, url: U, account: &Account) -> reqwest::RequestBuilder {
//         let builder = reqwest::Client::new().get(url);
//         crate::headers::add_account_auth_header(builder, account.jwt().to_string())
//     }
//
//     async fn get_account(
//         &self,
//         account: &Account,
//     ) -> Result<NymVpnAccountResponse, nym_http_api_client::HttpClientError> {
//         dbg!(&account.jwt());
//         // let url = self.path(&[routes::V1, routes::ACCOUNT, &account.id()]);
//         // let response = self.get_with_account(url, account).send().await?;
//         // dbg!(&response);
//
//         let response = self
//             .get_json(
//                 &[routes::V1, routes::ACCOUNT, &account.id()],
//                 nym_http_api_client::NO_PARAMS,
//             )
//             .await;
//
//         // let text = response.text().await.unwrap();
//         // dbg!(&text);
//
//         response
//
//         // let json = match response.status() {
//         //     StatusCode::OK => response.json().await.map_err(ApiError::from),
//         //     StatusCode::FORBIDDEN => Err(ApiError::Forbidden(response.json().await.unwrap())),
//         //     _ => Err(ApiError::Unknown),
//         // };
//         // dbg!(&json);
//         // json
//     }
// }

pub struct AccountClient2 {
    client: nym_http_api_client::Client,
}

impl AccountClient2 {
    fn new(base_url: Url) -> Self {
        let client = nym_http_api_client::Client::builder::<_,nym_http_api_client::HttpClientError>(base_url)
            .unwrap()
            // .with_user_agent(format!("nym-wasm-znym-lib/{}", env!("CARGO_PKG_VERSION")))
            .build::<nym_http_api_client::HttpClientError>()
            .unwrap();
        Self { client }
    }

    async fn get_account(
        &self,
        account: &Account,
        device: &Device,
    ) -> Result<NymVpnAccountResponse, nym_http_api_client::HttpClientError> {
        dbg!(&account.jwt());
        // let url = self.path(&[routes::V1, routes::ACCOUNT, &account.id()]);
        // let response = self.get_with_account(url, account).send().await?;
        // dbg!(&response);

        let mut headers = reqwest::header::HeaderMap::new();
        let auth_value = format!("Bearer {}", device.jwt());
        let mut header_value = reqwest::header::HeaderValue::from_str(&auth_value).unwrap();
        header_value.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, header_value);

        let request = self
            .client
            .create_get_request(
                &[routes::V1, routes::ACCOUNT, &account.id()],
                nym_http_api_client::NO_PARAMS,
            )
            // .headers(headers);
            .bearer_auth(account.jwt().to_string())
            .header(DEVICE_AUTHORIZATION_HEADER, device.jwt().to_string());
        // let request = crate::headers::add_account_auth_header(request, account.jwt().to_string());
        //
        // builder.header(DEVICE_AUTHORIZATION_HEADER, format!("Bearer {jwt}"))

        // Add header
        // let device_auth_header = reqwest::header::HeaderMap::new();
        // let mut headers = reqwest::header::HeaderMap::new();
        // let auth_value = format!("Bearer {}", device.jwt());
        // let mut header_value = reqwest::header::HeaderValue::from_str(&auth_value).unwrap();
        // header_value.set_sensitive(true);
        // headers.insert(reqwest::header::AUTHORIZATION, header_value);

        let response = request.send().await?;
        dbg!(&response);

        nym_http_api_client::parse_response(response, false).await

        // let text = response.text().await.unwrap();
        // dbg!(&text);

        // let json = match response.status() {
        //     StatusCode::OK => response.json().await.map_err(ApiError::from),
        //     StatusCode::FORBIDDEN => Err(ApiError::Forbidden(response.json().await.unwrap())),
        //     _ => Err(ApiError::Unknown),
        // };
        // dbg!(&json);
        // json
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
        // let client = AccountClient::new(BASE_URL.parse().unwrap());
        let client = AccountClient2::new(BASE_URL.parse().unwrap());
        // client.get_account(&account).await;
        let r = client.get_account(&account, &device).await;

        dbg!(&r);
    }

    // #[tokio::test]
    // async fn add_device() {
    //     let account = Account::from(get_mnemonic());
    //     let device = Device::from(get_ed25519_keypair());
    //     let client = AccountClient::new(BASE_URL.parse().unwrap());
    //     client.add_device(&account, &device).await;
    // }
}
