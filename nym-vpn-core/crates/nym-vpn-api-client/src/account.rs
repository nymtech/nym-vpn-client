// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::{HttpClientError, PathSegments, UserAgent, NO_PARAMS};
use request::{CreateSubscriptionRequestBody, RegisterDeviceRequestBody, RequestZkNymRequestBody};
use reqwest::Url;
use response::{
    NymDirectoryGatewayCountriesResponse, NymDirectoryGatewaysResponse, NymErrorResponse,
    NymVpnAccountResponse, NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnDevicesResponse,
    NymVpnSubscription, NymVpnSubscriptionResponse, NymVpnZkNym, NymVpnZkNymResponse,
    UnexpectedError,
};
use serde::{de::DeserializeOwned, Serialize};
use types::{Account, Device};

use crate::headers::DEVICE_AUTHORIZATION_HEADER;

pub mod types {
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
}

mod routes {
    pub const PUBLIC: &str = "public";
    pub const V1: &str = "v1";
    pub const ACCOUNT: &str = "account";
    pub const SUMMARY: &str = "summary";
    pub const DEVICE: &str = "device";
    pub const ACTIVE: &str = "active";
    pub const ZKNYM: &str = "zknym";
    pub const SUBSCRIPTION: &str = "subscription";
    pub const DIRECTORY: &str = "directory";
    pub const GATEWAYS: &str = "gateways";
    pub const COUNTRIES: &str = "countries";
    pub const ENTRY: &str = "entry";
    pub const EXIT: &str = "exit";
}

mod request {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RegisterDeviceRequestBody {
        pub device_identity_key: String,
        pub signature: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestZkNymRequestBody {
        pub blinded_signing_request_base58: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateSubscriptionRequestBody {
        pub valid_from_utc: String,
        pub subscription_kind: String,
    }
}

mod response {
    use std::fmt;

    use serde::{Deserialize, Serialize};

    use crate::{Country, Gateway};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnAccountResponse {
        created_on_utc: String,
        last_updated_utc: String,
        account_addr: String,
        status: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnAccountSummarySubscription {
        is_active: bool,
        active: Option<NymVpnSubscription>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnAccountSummaryDevices {
        active: u64,
        max: u64,
        remaining: u64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnAccountSummaryFairUsage {
        used_gb: f64,
        limit_gb: f64,
        resets_on_utc: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnAccountSummaryResponse {
        account: NymVpnAccountResponse,
        subscription: NymVpnAccountSummarySubscription,
        devices: NymVpnAccountSummaryDevices,
        fair_usage: NymVpnAccountSummaryFairUsage,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnDevice {
        created_on_utc: String,
        last_updated_utc: String,
        device_identity_key: String,
        status: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnDevicesResponse {
        total_items: u64,
        page: u64,
        page_size: u64,
        devices: Vec<NymVpnDevice>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnZkNym {
        created_on_utc: String,
        last_updated_utc: String,
        id: String,
        valid_until_utc: String,
        valid_from_utc: String,
        issued_bandwidth_in_gb: f64,
        blinded_shares: Vec<String>,
        status: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnZkNymResponse {
        total_items: u64,
        page: u64,
        page_size: u64,
        zk_nyms: Vec<NymVpnZkNym>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnSubscription {
        created_on_utc: String,
        last_updated_utc: String,
        id: String,
        valid_until_utc: String,
        valid_from_utc: String,
        status: String,
        kind: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymVpnSubscriptionResponse {
        total_items: u64,
        page: u64,
        page_size: u64,
        subscriptions: Vec<NymVpnSubscription>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymDirectoryGatewaysResponse(Vec<Gateway>);

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NymDirectoryGatewayCountriesResponse(Vec<Country>);

    #[derive(Debug, Serialize, Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct NymErrorResponse {
        pub message: String,
        pub message_id: Option<String>,
        pub code_reference_id: Option<String>,
        pub status: String,
    }

    impl fmt::Display for NymErrorResponse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let fields = [
                Some(format!("message: {}", self.message)),
                self.message_id
                    .as_deref()
                    .map(|x| format!("message_id: {}", x)),
                self.code_reference_id
                    .as_deref()
                    .map(|x| format!("code_reference_id: {}", x)),
                Some(format!("status: {}", self.status)),
            ]
            .iter()
            .filter_map(|x| x.clone())
            .collect::<Vec<_>>();
            write!(f, "{}", fields.join(", "))
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UnexpectedError {
        pub message: String,
    }

    impl fmt::Display for UnexpectedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }
}

pub struct AccountClient {
    inner: nym_http_api_client::Client,
}

impl AccountClient {
    pub fn new(base_url: Url, user_agent: UserAgent) -> Result<Self, HttpClientError> {
        let inner = nym_http_api_client::Client::builder(base_url)?
            .with_user_agent(user_agent)
            .build()?;
        Ok(Self { inner })
    }

    async fn get<T>(
        &self,
        path: PathSegments<'_>,
        account: &Account,
        device: Option<&Device>,
    ) -> Result<T, HttpClientError<NymErrorResponse>>
    where
        T: DeserializeOwned,
    {
        let request = self
            .inner
            .create_get_request(path, NO_PARAMS)
            .bearer_auth(account.jwt().to_string());

        let request = match device {
            Some(device) => request.header(DEVICE_AUTHORIZATION_HEADER, device.jwt().to_string()),
            None => request,
        };

        let response = request.send().await?;

        nym_http_api_client::parse_response(response, false).await
    }

    async fn post<T, B>(
        &self,
        path: PathSegments<'_>,
        json_body: &B,
        account: &Account,
        device: Option<&Device>,
    ) -> Result<T, HttpClientError<NymErrorResponse>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self
            .inner
            .create_post_request(path, NO_PARAMS, json_body)
            .bearer_auth(account.jwt().to_string());

        let request = match device {
            Some(device) => request.header(DEVICE_AUTHORIZATION_HEADER, device.jwt().to_string()),
            None => request,
        };

        let response = request.send().await?;

        nym_http_api_client::parse_response(response, false).await
    }

    // ACCOUNT

    pub async fn get_account(
        &self,
        account: &Account,
    ) -> Result<NymVpnAccountResponse, HttpClientError<NymErrorResponse>> {
        self.get(
            &[routes::PUBLIC, routes::V1, routes::ACCOUNT, &account.id()],
            account,
            None,
        )
        .await
    }

    pub async fn get_account_summary(
        &self,
        account: &Account,
    ) -> Result<NymVpnAccountSummaryResponse, HttpClientError<NymErrorResponse>> {
        self.get(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::SUMMARY,
            ],
            account,
            None,
        )
        .await
    }

    // DEVICES

    pub async fn get_devices(
        &self,
        account: &Account,
    ) -> Result<NymVpnDevicesResponse, HttpClientError<NymErrorResponse>> {
        self.get(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
            ],
            account,
            None,
        )
        .await
    }

    pub async fn register_device(
        &self,
        account: &Account,
        device: &Device,
    ) -> Result<NymVpnDevice, HttpClientError<NymErrorResponse>> {
        let body = RegisterDeviceRequestBody {
            device_identity_key: device.identity_key().to_base58_string(),
            signature: device.jwt().to_string(),
        };

        self.post(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
            ],
            &body,
            account,
            Some(device),
        )
        .await
    }

    pub async fn get_active_devices(
        &self,
        account: &Account,
    ) -> Result<NymVpnDevicesResponse, HttpClientError<NymErrorResponse>> {
        self.get(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
                routes::ACTIVE,
            ],
            account,
            None,
        )
        .await
    }

    pub async fn get_device_by_id(
        &self,
        account: &Account,
        device: &Device,
    ) -> Result<NymVpnDevice, HttpClientError<NymErrorResponse>> {
        self.get(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
                &device.identity_key().to_string(),
            ],
            account,
            None,
        )
        .await
    }

    // ZK-NYM

    pub async fn get_device_zk_nyms(
        &self,
        account: &Account,
        device: &Device,
    ) -> Result<NymVpnZkNymResponse, HttpClientError<NymErrorResponse>> {
        self.get(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
                &device.identity_key().to_string(),
                routes::ZKNYM,
            ],
            account,
            Some(device),
        )
        .await
    }

    pub async fn request_zk_nym(
        &self,
        account: &Account,
        device: &Device,
    ) -> Result<NymVpnZkNym, HttpClientError<NymErrorResponse>> {
        let body = RequestZkNymRequestBody {
            blinded_signing_request_base58: "todo".to_string(),
        };

        self.post(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
                &device.identity_key().to_string(),
                routes::ZKNYM,
            ],
            &body,
            account,
            Some(device),
        )
        .await
    }

    pub async fn get_active_zk_nym(
        &self,
        account: &Account,
        device: &Device,
    ) -> Result<NymVpnZkNym, HttpClientError<NymErrorResponse>> {
        self.get(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
                &device.identity_key().to_string(),
                routes::ZKNYM,
                routes::ACTIVE,
            ],
            account,
            Some(device),
        )
        .await
    }

    pub async fn get_zk_nym_by_id(
        &self,
        account: &Account,
        device: &Device,
        id: &str,
    ) -> Result<NymVpnZkNym, HttpClientError<NymErrorResponse>> {
        self.get(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
                &device.identity_key().to_string(),
                routes::ZKNYM,
                id,
            ],
            account,
            Some(device),
        )
        .await
    }

    // SUBSCRIPTIONS

    pub async fn get_subscriptions(
        &self,
        account: &Account,
    ) -> Result<NymVpnSubscriptionResponse, HttpClientError<NymErrorResponse>> {
        self.get(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::SUBSCRIPTION,
            ],
            account,
            None,
        )
        .await
    }

    pub async fn create_subscription(
        &self,
        account: &Account,
    ) -> Result<NymVpnSubscription, HttpClientError<NymErrorResponse>> {
        let body = CreateSubscriptionRequestBody {
            valid_from_utc: "todo".to_string(),
            subscription_kind: "todo".to_string(),
        };

        self.post(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::SUBSCRIPTION,
            ],
            &body,
            account,
            None,
        )
        .await
    }

    pub async fn get_active_subscriptions(
        &self,
        account: &Account,
    ) -> Result<NymVpnSubscriptionResponse, HttpClientError<NymErrorResponse>> {
        self.get(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::SUBSCRIPTION,
                routes::ACTIVE,
            ],
            account,
            None,
        )
        .await
    }

    // GATEWAYS

    pub async fn get_gateways(
        &self,
    ) -> Result<NymDirectoryGatewaysResponse, HttpClientError<UnexpectedError>> {
        self.inner
            .get_json(
                &[
                    // Add public/v1 to the path once the api is updated
                    //routes::PUBLIC,
                    //routes::V1,
                    routes::DIRECTORY,
                    routes::GATEWAYS,
                ],
                NO_PARAMS,
            )
            .await
    }

    pub async fn get_gateway_countries(
        &self,
    ) -> Result<NymDirectoryGatewayCountriesResponse, HttpClientError<UnexpectedError>> {
        self.inner
            .get_json(
                &[
                    // Add public/v1 to the path once the api is updated
                    //routes::PUBLIC,
                    //routes::V1,
                    routes::DIRECTORY,
                    routes::GATEWAYS,
                    routes::COUNTRIES,
                ],
                NO_PARAMS,
            )
            .await
    }

    pub async fn get_entry_gateways(
        &self,
    ) -> Result<NymDirectoryGatewaysResponse, HttpClientError<UnexpectedError>> {
        self.inner
            .get_json(
                &[
                    // Add public/v1 to the path once the api is updated
                    //routes::PUBLIC,
                    //routes::V1,
                    routes::DIRECTORY,
                    routes::GATEWAYS,
                    routes::ENTRY,
                ],
                NO_PARAMS,
            )
            .await
    }

    pub async fn get_entry_gateway_countries(
        &self,
    ) -> Result<NymDirectoryGatewayCountriesResponse, HttpClientError<UnexpectedError>> {
        self.inner
            .get_json(
                &[
                    // Add public/v1 to the path once the api is updated
                    //routes::PUBLIC,
                    //routes::V1,
                    routes::DIRECTORY,
                    routes::GATEWAYS,
                    routes::ENTRY,
                    routes::COUNTRIES,
                ],
                NO_PARAMS,
            )
            .await
    }

    pub async fn get_exit_gateways(
        &self,
    ) -> Result<NymDirectoryGatewaysResponse, HttpClientError<UnexpectedError>> {
        self.inner
            .get_json(
                &[
                    // Add public/v1 to the path once the api is updated
                    //routes::PUBLIC,
                    //routes::V1,
                    routes::DIRECTORY,
                    routes::GATEWAYS,
                    routes::EXIT,
                ],
                NO_PARAMS,
            )
            .await
    }

    pub async fn get_exit_gateway_countries(
        &self,
    ) -> Result<NymDirectoryGatewayCountriesResponse, HttpClientError<UnexpectedError>> {
        self.inner
            .get_json(
                &[
                    // Add public/v1 to the path once the api is updated
                    //routes::PUBLIC,
                    //routes::V1,
                    routes::DIRECTORY,
                    routes::GATEWAYS,
                    routes::EXIT,
                    routes::COUNTRIES,
                ],
                NO_PARAMS,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nym_crypto::asymmetric::ed25519;

    // const BASE_URL: &str = "https://nymvpn.com/api";
    const BASE_URL: &str =
        "https://nym-dot-com-git-deploy-canary-nyx-network-staging.vercel.app/api";

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

    fn user_agent() -> UserAgent {
        UserAgent {
            version: "0.1.0".to_string(),
            application: "nym".to_string(),
            platform: "linux".to_string(),
            git_commit: "123456".to_string(),
        }
    }

    #[tokio::test]
    async fn get_account() {
        let account = Account::from(get_mnemonic());
        let client = AccountClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
        let r = client.get_account(&account).await;
        dbg!(&r);
        println!("{}", r.unwrap_err());
    }

    #[tokio::test]
    async fn get_device_zk_nyms() {
        let account = Account::from(get_mnemonic());
        let device = Device::from(get_ed25519_keypair());
        let client = AccountClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
        let r = client.get_device_zk_nyms(&account, &device).await;
        dbg!(&r);
    }

    #[tokio::test]
    async fn get_gateways() {
        let client = AccountClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
        let r = client.get_gateways().await;
        dbg!(&r);
    }

    #[tokio::test]
    async fn get_gateway_countries() {
        let client = AccountClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
        let r = client.get_gateway_countries().await;
        dbg!(&r);
    }
}
