// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, time::Duration};

use backon::Retryable;
use nym_http_api_client::{HttpClientError, Params, PathSegments, UserAgent, NO_PARAMS};
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    error::{Result, VpnApiClientError},
    request::{
        ApplyFreepassRequestBody, CreateSubscriptionKind, CreateSubscriptionRequestBody,
        RegisterDeviceRequestBody, RequestZkNymRequestBody,
    },
    response::{
        NymDirectoryGatewayCountriesResponse, NymDirectoryGatewaysResponse, NymVpnAccountResponse,
        NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnDevicesResponse, NymVpnSubscription,
        NymVpnSubscriptionResponse, NymVpnSubscriptionsResponse, NymVpnZkNym, NymVpnZkNymResponse,
    },
    routes,
    types::{Device, GatewayMinPerformance, GatewayType, VpnApiAccount},
};

pub(crate) const DEVICE_AUTHORIZATION_HEADER: &str = "x-device-authorization";

// GET requests can unfortunately take a long time over the mixnet
pub(crate) const NYM_VPN_API_TIMEOUT: Duration = Duration::from_secs(60);

pub struct VpnApiClient {
    inner: nym_http_api_client::Client,
}

impl VpnApiClient {
    pub fn new(base_url: Url, user_agent: UserAgent) -> Result<Self> {
        nym_http_api_client::Client::builder(base_url)
            .map(|builder| {
                builder
                    .with_user_agent(user_agent)
                    .with_timeout(NYM_VPN_API_TIMEOUT)
            })
            .and_then(|builder| builder.build())
            .map(|c| Self { inner: c })
            .map_err(VpnApiClientError::FailedToCreateVpnApiClient)
    }

    async fn get_authorized<T, E>(
        &self,
        path: PathSegments<'_>,
        account: &VpnApiAccount,
        device: Option<&Device>,
    ) -> std::result::Result<T, HttpClientError<E>>
    where
        T: DeserializeOwned,
        E: fmt::Display + DeserializeOwned,
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

    async fn get_json_with_retry<T, K, V, E>(
        &self,
        path: PathSegments<'_>,
        params: Params<'_, K, V>,
    ) -> std::result::Result<T, HttpClientError<E>>
    where
        for<'a> T: Deserialize<'a>,
        K: AsRef<str>,
        V: AsRef<str>,
        E: fmt::Display + fmt::Debug + DeserializeOwned,
    {
        let response = (|| async { self.inner.get_json(path, params).await })
            .retry(backon::ConstantBuilder::default())
            .notify(|err: &HttpClientError<E>, dur: Duration| {
                tracing::warn!("Failed to get JSON: {}", err);
                tracing::warn!("retrying {:?} after {:?}", err, dur);
            })
            .await?;
        Ok(response)
    }

    async fn post_authorized<T, B, E>(
        &self,
        path: PathSegments<'_>,
        json_body: &B,
        account: &VpnApiAccount,
        device: Option<&Device>,
    ) -> std::result::Result<T, HttpClientError<E>>
    where
        T: DeserializeOwned,
        B: Serialize,
        E: fmt::Display + DeserializeOwned,
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

    pub async fn get_account(&self, account: &VpnApiAccount) -> Result<NymVpnAccountResponse> {
        self.get_authorized(
            &[routes::PUBLIC, routes::V1, routes::ACCOUNT, &account.id()],
            account,
            None,
        )
        .await
        .map_err(crate::error::VpnApiClientError::FailedToGetAccount)
    }

    pub async fn get_account_summary(
        &self,
        account: &VpnApiAccount,
    ) -> Result<NymVpnAccountSummaryResponse> {
        self.get_authorized(
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
        .map_err(VpnApiClientError::FailedToGetAccountSummary)
    }

    // DEVICES

    pub async fn get_devices(&self, account: &VpnApiAccount) -> Result<NymVpnDevicesResponse> {
        self.get_authorized(
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
        .map_err(VpnApiClientError::FailedToGetDevices)
    }

    pub async fn register_device(
        &self,
        account: &VpnApiAccount,
        device: &Device,
    ) -> Result<NymVpnDevice> {
        let body = RegisterDeviceRequestBody {
            device_identity_key: device.identity_key().to_base58_string(),
            signature: device.sign_identity_key().to_base64_string(),
        };

        self.post_authorized(
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
        .map_err(VpnApiClientError::FailedToRegisterDevice)
    }

    pub async fn get_active_devices(
        &self,
        account: &VpnApiAccount,
    ) -> Result<NymVpnDevicesResponse> {
        self.get_authorized(
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
        .map_err(VpnApiClientError::FailedToGetActiveDevices)
    }

    pub async fn get_device_by_id(
        &self,
        account: &VpnApiAccount,
        device: &Device,
    ) -> Result<NymVpnDevice> {
        self.get_authorized(
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
        .map_err(VpnApiClientError::FailedToGetDeviceById)
    }

    // ZK-NYM

    pub async fn get_device_zk_nyms(
        &self,
        account: &VpnApiAccount,
        device: &Device,
    ) -> Result<NymVpnZkNymResponse> {
        self.get_authorized(
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
        .map_err(VpnApiClientError::FailedToGetDeviceZkNyms)
    }

    pub async fn request_zk_nym(
        &self,
        account: &VpnApiAccount,
        device: &Device,
    ) -> Result<NymVpnZkNym> {
        let body = RequestZkNymRequestBody {
            withdrawal_request: "todo!".to_string(),
            ecash_pubkey: "todo!".to_string(),
            ticketbook_type: "todo!".to_string(),
        };

        self.post_authorized(
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
        .map_err(VpnApiClientError::FailedToRequestZkNym)
    }

    pub async fn get_active_zk_nym(
        &self,
        account: &VpnApiAccount,
        device: &Device,
    ) -> Result<NymVpnZkNym> {
        self.get_authorized(
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
        .map_err(VpnApiClientError::FailedToGetActiveZkNym)
    }

    pub async fn get_zk_nym_by_id(
        &self,
        account: &VpnApiAccount,
        device: &Device,
        id: &str,
    ) -> Result<NymVpnZkNym> {
        self.get_authorized(
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
        .map_err(VpnApiClientError::FailedToGetZkNymById)
    }

    // FREEPASS

    pub async fn get_free_passes(
        &self,
        account: &VpnApiAccount,
    ) -> Result<NymVpnSubscriptionsResponse> {
        self.get_authorized(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::FREEPASS,
            ],
            account,
            None,
        )
        .await
        .map_err(VpnApiClientError::FailedToGetFreePasses)
    }

    pub async fn apply_freepass(
        &self,
        account: &VpnApiAccount,
        code: String,
    ) -> Result<NymVpnSubscription> {
        let body = ApplyFreepassRequestBody { code };

        self.post_authorized(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::FREEPASS,
            ],
            &body,
            account,
            None,
        )
        .await
        .map_err(VpnApiClientError::FailedToApplyFreepass)
    }

    // SUBSCRIPTIONS

    pub async fn get_subscriptions(
        &self,
        account: &VpnApiAccount,
    ) -> Result<NymVpnSubscriptionsResponse> {
        self.get_authorized(
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
        .map_err(VpnApiClientError::FailedToGetSubscriptions)
    }

    pub async fn create_subscription(&self, account: &VpnApiAccount) -> Result<NymVpnSubscription> {
        let body = CreateSubscriptionRequestBody {
            valid_from_utc: "todo".to_string(),
            subscription_kind: CreateSubscriptionKind::OneMonth,
        };

        self.post_authorized(
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
        .map_err(VpnApiClientError::FailedToCreateSubscription)
    }

    pub async fn get_active_subscriptions(
        &self,
        account: &VpnApiAccount,
    ) -> Result<NymVpnSubscriptionResponse> {
        self.get_authorized(
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
        .map_err(VpnApiClientError::FailedToGetActiveSubscriptions)
    }

    // GATEWAYS

    pub async fn get_gateways(
        &self,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewaysResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::GATEWAYS,
            ],
            &min_performance.unwrap_or_default().to_param(),
        )
        .await
        .map_err(VpnApiClientError::FailedToGetGateways)
    }

    pub async fn get_gateways_by_type(
        &self,
        kind: GatewayType,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewaysResponse> {
        match kind {
            GatewayType::MixnetEntry => self.get_entry_gateways(min_performance).await,
            GatewayType::MixnetExit => self.get_exit_gateways(min_performance).await,
            GatewayType::Wg => self.get_vpn_gateways(min_performance).await,
        }
    }

    pub async fn get_gateway_countries_by_type(
        &self,
        kind: GatewayType,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewayCountriesResponse> {
        match kind {
            GatewayType::MixnetEntry => self.get_entry_gateway_countries(min_performance).await,
            GatewayType::MixnetExit => self.get_exit_gateway_countries(min_performance).await,
            GatewayType::Wg => self.get_vpn_gateway_countries(min_performance).await,
        }
    }

    pub async fn get_vpn_gateways(
        &self,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewaysResponse> {
        let mut params = min_performance.unwrap_or_default().to_param();
        params.push((routes::SHOW_VPN_ONLY.to_string(), "true".to_string()));
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::GATEWAYS,
            ],
            &params,
        )
        .await
        .map_err(VpnApiClientError::FailedToGetVpnGateways)
    }

    pub async fn get_vpn_gateway_countries(
        &self,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewayCountriesResponse> {
        let mut params = min_performance.unwrap_or_default().to_param();
        params.push((routes::SHOW_VPN_ONLY.to_string(), "true".to_string()));
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::GATEWAYS,
                routes::COUNTRIES,
            ],
            &params,
        )
        .await
        .map_err(VpnApiClientError::FailedToGetVpnGatewayCountries)
    }

    pub async fn get_gateway_countries(
        &self,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewayCountriesResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::GATEWAYS,
                routes::COUNTRIES,
            ],
            &min_performance.unwrap_or_default().to_param(),
        )
        .await
        .map_err(VpnApiClientError::FailedToGetGatewayCountries)
    }

    pub async fn get_entry_gateways(
        &self,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewaysResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::GATEWAYS,
                routes::ENTRY,
            ],
            &min_performance.unwrap_or_default().to_param(),
        )
        .await
        .map_err(VpnApiClientError::FailedToGetEntryGateways)
    }

    pub async fn get_entry_gateway_countries(
        &self,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewayCountriesResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::GATEWAYS,
                routes::ENTRY,
                routes::COUNTRIES,
            ],
            &min_performance.unwrap_or_default().to_param(),
        )
        .await
        .map_err(VpnApiClientError::FailedToGetEntryGatewayCountries)
    }

    pub async fn get_exit_gateways(
        &self,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewaysResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::GATEWAYS,
                routes::EXIT,
            ],
            &min_performance.unwrap_or_default().to_param(),
        )
        .await
        .map_err(VpnApiClientError::FailedToGetExitGateways)
    }

    pub async fn get_exit_gateway_countries(
        &self,
        min_performance: Option<GatewayMinPerformance>,
    ) -> Result<NymDirectoryGatewayCountriesResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::GATEWAYS,
                routes::EXIT,
                routes::COUNTRIES,
            ],
            &min_performance.unwrap_or_default().to_param(),
        )
        .await
        .map_err(VpnApiClientError::FailedToGetExitGatewayCountries)
    }
}

#[cfg(test)]
mod tests {
    use nym_crypto::asymmetric::ed25519;

    use super::*;

    const BASE_URL: &str = "https://nymvpn.com/api";

    fn user_agent() -> UserAgent {
        UserAgent {
            version: "0.1.0".to_string(),
            application: "nym".to_string(),
            platform: "linux".to_string(),
            git_commit: "123456".to_string(),
        }
    }

    mod account {
        use super::*;

        // Preview deployment example data
        struct PreviewData {
            base_url: &'static str,
            account_mnemonic: &'static str,
            device_private_key_base58: &'static str,
            device_public_key_base58: &'static str,
        }

        fn preview_data() -> PreviewData {
            #[allow(unreachable_code)]
            PreviewData {
                base_url: todo!(),
                account_mnemonic: todo!(),
                device_private_key_base58: todo!(),
                device_public_key_base58: todo!(),
            }
        }

        fn base_url_preview() -> Url {
            preview_data().base_url.parse().unwrap()
        }

        fn get_mnemonic() -> bip39::Mnemonic {
            preview_data().account_mnemonic.parse().unwrap()
        }

        fn get_ed25519_keypair() -> ed25519::KeyPair {
            let private_key_base58 = preview_data().device_private_key_base58;
            let public_key_base58 = preview_data().device_public_key_base58;

            let private_key = bs58::decode(private_key_base58).into_vec().unwrap();
            let public_key = bs58::decode(public_key_base58).into_vec().unwrap();

            ed25519::KeyPair::from_bytes(&private_key, &public_key).unwrap()
        }

        // These tests are all iffy since we are running against a preview deployment, but they are
        // useful to drive implementetion and to check that the API is working as expected.

        #[ignore]
        #[tokio::test]
        async fn get_account() {
            let account = VpnApiAccount::from(get_mnemonic());
            let client = VpnApiClient::new(base_url_preview(), user_agent()).unwrap();
            let response = client.get_account(&account).await.unwrap();
            dbg!(&response);
        }

        #[ignore]
        #[tokio::test]
        async fn get_account_summary() {
            let account = VpnApiAccount::from(get_mnemonic());
            let client = VpnApiClient::new(base_url_preview(), user_agent()).unwrap();
            let response = client.get_account_summary(&account).await.unwrap();
            dbg!(&response);
        }

        #[ignore]
        #[tokio::test]
        async fn get_devices() {
            let account = VpnApiAccount::from(get_mnemonic());
            let client = VpnApiClient::new(base_url_preview(), user_agent()).unwrap();
            let response = client.get_devices(&account).await.unwrap();
            dbg!(&response);
        }

        #[ignore]
        #[tokio::test]
        async fn get_device_zk_nyms() {
            let account = VpnApiAccount::from(get_mnemonic());
            let device = Device::from(get_ed25519_keypair());
            let client = VpnApiClient::new(base_url_preview(), user_agent()).unwrap();
            let response = client.get_device_zk_nyms(&account, &device).await;
            dbg!(&response);
        }
    }

    mod gateway_directory {
        use super::*;

        #[tokio::test]
        async fn get_gateways() {
            let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
            let response = client
                .get_gateways(Some(GatewayMinPerformance::default()))
                .await
                .unwrap();
            assert!(!response.into_inner().is_empty());
        }

        #[tokio::test]
        async fn get_entry_gateways() {
            let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
            let response = client.get_entry_gateways(None).await.unwrap();
            assert!(!response.into_inner().is_empty());
        }

        #[tokio::test]
        async fn get_exit_gateways() {
            let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
            let response = client.get_entry_gateways(None).await.unwrap();
            assert!(!response.into_inner().is_empty());
        }

        #[tokio::test]
        async fn get_gateway_countries() {
            let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
            let response = client.get_gateway_countries(None).await.unwrap();
            assert!(!response.into_inner().is_empty());
        }

        #[tokio::test]
        async fn get_entry_gateway_countries() {
            let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
            let response = client.get_entry_gateway_countries(None).await.unwrap();
            assert!(!response.into_inner().is_empty());
        }

        #[tokio::test]
        async fn get_exit_gateway_countries() {
            let client = VpnApiClient::new(BASE_URL.parse().unwrap(), user_agent()).unwrap();
            let response = client.get_exit_gateway_countries(None).await.unwrap();
            assert!(!response.into_inner().is_empty());
        }
    }
}
