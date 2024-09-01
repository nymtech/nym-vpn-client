// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::{HttpClientError, PathSegments, UserAgent, NO_PARAMS};
use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::VpnApiClientError,
    headers::DEVICE_AUTHORIZATION_HEADER,
    request::{CreateSubscriptionRequestBody, RegisterDeviceRequestBody, RequestZkNymRequestBody},
    responses::{
        NymDirectoryGatewayCountriesResponse, NymDirectoryGatewaysResponse, NymErrorResponse,
        NymVpnAccountResponse, NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnDevicesResponse,
        NymVpnSubscription, NymVpnSubscriptionResponse, NymVpnZkNym, NymVpnZkNymResponse,
        UnexpectedError,
    },
    routes,
    types::{Account, Device},
};

pub struct VpnApiClient {
    inner: nym_http_api_client::Client,
}

impl VpnApiClient {
    // pub fn new(base_url: Url, user_agent: UserAgent) -> Result<Self, HttpClientError> {
    pub fn new(base_url: Url, user_agent: UserAgent) -> Result<Self, VpnApiClientError> {
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
        let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
        let r = client.get_account(&account).await;
        dbg!(&r);
        println!("{}", r.unwrap_err());
    }

    #[tokio::test]
    async fn get_device_zk_nyms() {
        let account = Account::from(get_mnemonic());
        let device = Device::from(get_ed25519_keypair());
        let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
        let r = client.get_device_zk_nyms(&account, &device).await;
        dbg!(&r);
    }

    #[tokio::test]
    async fn get_gateways() {
        let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
        let r = client.get_gateways().await;
        dbg!(&r);
    }

    #[tokio::test]
    async fn get_gateway_countries() {
        let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
        let r = client.get_gateway_countries().await;
        dbg!(&r);
    }
}
