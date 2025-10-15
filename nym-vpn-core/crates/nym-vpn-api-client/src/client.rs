// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use backon::Retryable;
use nym_credential_proxy_requests::api::v1::ticketbook::models::PartialVerificationKeysResponse;
use nym_http_api_client::{
    ApiClient, Client, HttpClientError, NO_PARAMS, Params, PathSegments, Url, UserAgent,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use time::OffsetDateTime;

use crate::{
    ResolverOverrides, api_urls_to_urls,
    error::{Result, VpnApiClientError},
    fronted_http_client,
    request::{
        ApplyFreepassRequestBody, CreateAndroidAccountRequestBody, CreateAppleAccountRequestBody,
        CreateSubscriptionKind, CreateSubscriptionRequestBody, RegisterDeviceRequestBody,
        RequestZkNymRequestBody, UpdateDeviceRequestBody, UpdateDeviceRequestStatus,
    },
    response::{
        NymDirectoryGatewayCountriesResponse, NymDirectoryGatewaysResponse, NymVpnAccountResponse,
        NymVpnAccountSummaryResponse, NymVpnAccountSummaryWithDeviceResponse, NymVpnDevice,
        NymVpnDevicesResponse, NymVpnHealthResponse, NymVpnRegisterAccountResponse,
        NymVpnSubscription, NymVpnSubscriptionResponse, NymVpnSubscriptionsResponse,
        NymVpnUsagesResponse, NymVpnZkNym, NymVpnZkNymPost, NymVpnZkNymResponse,
        NymWellknownDiscoveryItem, StatusOk,
    },
    routes,
    types::{
        Device, DeviceStatus, GatewayMinPerformance, GatewayType, Platform, VpnAccount, VpnApiTime,
        VpnApiTimeSynced,
    },
};

pub(crate) const DEVICE_AUTHORIZATION_HEADER: &str = "x-device-authorization";

// GET requests can unfortunately take a long time over the mixnet
pub(crate) const NYM_VPN_API_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug)]
pub struct VpnApiClient {
    inner: Client,
    urls: Vec<Url>,
    user_agent: UserAgent,
}

impl VpnApiClient {
    pub async fn new(
        urls: Vec<Url>,
        user_agent: UserAgent,
        resolver_overrides: Option<&ResolverOverrides>,
    ) -> Result<Self> {
        let inner = fronted_http_client(
            urls.clone(),
            Some(user_agent.clone()),
            Some(NYM_VPN_API_TIMEOUT),
            resolver_overrides,
        )
        .await?;

        Ok(Self {
            inner,
            urls,
            user_agent,
        })
    }

    #[cfg(feature = "network-defaults")]
    pub async fn from_network(
        network: &nym_network_defaults::NymNetworkDetails,
        user_agent: UserAgent,
        resolver_overrides: Option<&ResolverOverrides>,
    ) -> Result<Self> {
        #[allow(deprecated)]
        let api_urls = network.nym_vpn_api_urls.as_ref().ok_or_else(|| {
            let err: HttpClientError = HttpClientError::GenericRequestFailure(
                "No Nym VPN API URLs configured in network details".to_string(),
            );
            VpnApiClientError::CreateVpnApiClient(Box::new(err))
        })?;

        let urls = api_urls_to_urls(api_urls)?;

        let inner = fronted_http_client(
            urls.clone(),
            Some(user_agent.clone()),
            Some(NYM_VPN_API_TIMEOUT),
            resolver_overrides,
        )
        .await?;

        Ok(Self {
            inner,
            urls,
            user_agent,
        })
    }

    pub async fn override_resolver(
        &mut self,
        resolver_overrides: Option<&ResolverOverrides>,
    ) -> Result<()> {
        self.inner = fronted_http_client(
            self.urls.clone(),
            Some(self.user_agent.clone()),
            Some(NYM_VPN_API_TIMEOUT),
            resolver_overrides,
        )
        .await?;

        Ok(())
    }

    pub fn api_client(&self) -> &impl ApiClient {
        &self.inner
    }

    pub fn current_url(&self) -> &url::Url {
        self.inner.current_url().as_ref()
    }

    pub async fn get_remote_time(&self) -> Result<VpnApiTime> {
        let time_before = OffsetDateTime::now_utc();
        let remote_timestamp = self.get_health().await?.timestamp_utc;
        let time_after = OffsetDateTime::now_utc();

        Ok(VpnApiTime::from_remote_timestamp(
            time_before,
            remote_timestamp,
            time_after,
        ))
    }

    fn use_remote_time(remote_time: VpnApiTime) -> bool {
        match remote_time.is_synced() {
            VpnApiTimeSynced::AlmostSame => {
                tracing::debug!("{remote_time}");
                false
            }
            VpnApiTimeSynced::AcceptableSynced => {
                tracing::info!("{remote_time}");
                false
            }
            VpnApiTimeSynced::NotSynced => {
                tracing::warn!(
                    "The time skew between the local and remote time is too large, we'll use remote instead for JWT ({remote_time})."
                );
                true
            }
        }
    }

    async fn sync_with_remote_time(&self) -> Result<Option<VpnApiTime>> {
        let remote_time = self.get_remote_time().await?;

        if Self::use_remote_time(remote_time) {
            Ok(Some(remote_time))
        } else {
            Ok(None)
        }
    }

    async fn get_query<T>(
        &self,
        path: PathSegments<'_>,
        account: &VpnAccount,
        device: Option<&Device>,
        jwt: Option<VpnApiTime>,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        let request = self
            .inner
            .create_get_request(path, NO_PARAMS)?
            .bearer_auth(account.jwt(jwt).to_string());

        let request = match device {
            Some(device) => request.header(
                DEVICE_AUTHORIZATION_HEADER,
                format!("Bearer {}", device.jwt(jwt)),
            ),
            None => request,
        };
        let response = request.send().await?;
        nym_http_api_client::parse_response(response, false).await
    }

    async fn get_authorized<T>(
        &self,
        path: PathSegments<'_>,
        account: &VpnAccount,
        device: Option<&Device>,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        match self.get_query::<T>(path, account, device, None).await {
            Ok(response) => Ok(response),
            Err(err) => {
                if let HttpClientError::EndpointFailure { error, .. } = &err
                    && jwt_error(&error.to_string())
                {
                    tracing::warn!(
                        "Encountered possible JWT error: {error}. Retrying query with remote time"
                    );
                    if let Ok(Some(jwt)) = self.sync_with_remote_time().await.inspect_err(|err| {
                        tracing::error!("Failed to get remote time: {err}. Not retring anymore")
                    }) {
                        // retry with remote vpn api time, and return that only if it succeeds,
                        // otherwise return the initial error
                        let res = self.get_query(path, account, device, Some(jwt)).await;
                        if res.is_ok() {
                            return res;
                        }
                    }
                }
                Err(err)
            }
        }
    }

    #[allow(unused)]
    async fn get_authorized_debug<T>(
        &self,
        path: PathSegments<'_>,
        account: &VpnAccount,
        device: Option<&Device>,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        let request = self
            .inner
            .create_get_request(path, NO_PARAMS)?
            .bearer_auth(account.jwt(None).to_string());

        let request = match device {
            Some(device) => request.header(
                DEVICE_AUTHORIZATION_HEADER,
                format!("Bearer {}", device.jwt(None)),
            ),
            None => request,
        };

        let response = request.send().await?;
        let status = response.status();
        let headers = response.headers().clone();
        let url = response.url().clone();
        tracing::info!("Response status: {:#?}", status);

        // TODO: support this mode in the upstream crate

        let response_text = response.text().await.map(|t| t.to_owned());

        match response_text {
            Ok(response_text) => {
                if status.is_success() {
                    tracing::info!("Response: {:#?}", response_text);
                    #[allow(deprecated)]
                    let response_json = serde_json::from_str(&response_text)
                        .map_err(|e| HttpClientError::GenericRequestFailure(e.to_string()))?;
                    Ok(response_json)
                } else {
                    tracing::info!("Response: {:#?}", response_text);

                    Err(HttpClientError::EndpointFailure {
                        url,
                        status,
                        headers: Box::new(headers),
                        error: response_text,
                    })
                }
            }
            Err(err) => Err(HttpClientError::RequestFailure {
                url,
                status,
                headers: Box::new(headers),
            }),
        }
    }

    async fn get_json_with_retry<T, K, V>(
        &self,
        path: PathSegments<'_>,
        params: Params<'_, K, V>,
    ) -> std::result::Result<T, HttpClientError>
    where
        for<'a> T: Deserialize<'a>,
        K: AsRef<str> + Sync,
        V: AsRef<str> + Sync,
    {
        let response = (|| async { self.inner.get_json(path, params).await })
            .retry(backon::ConstantBuilder::default())
            .notify(|err: &HttpClientError, dur: Duration| {
                tracing::warn!("Failed to get JSON: {}", err);
                tracing::warn!("retrying after {:?}", dur);
            })
            .await?;
        Ok(response)
    }

    async fn post_json_with_retry<B, T, K, V>(
        &self,
        path: PathSegments<'_>,
        params: Params<'_, K, V>,
        json_body: &B,
    ) -> std::result::Result<T, HttpClientError>
    where
        for<'a> T: Deserialize<'a>,
        B: Serialize + ?Sized + Sync,
        K: AsRef<str> + Sync,
        V: AsRef<str> + Sync,
    {
        let response = (|| async { self.inner.post_json(path, params, json_body).await })
            .retry(backon::ConstantBuilder::default())
            .notify(|err: &HttpClientError, dur: Duration| {
                tracing::warn!("Failed to post JSON: {}", err);
                tracing::warn!("retrying after {:?}", dur);
            })
            .await?;
        Ok(response)
    }

    async fn post_query<T, B>(
        &self,
        path: PathSegments<'_>,
        json_body: &B,
        account: &VpnAccount,
        device: Option<&Device>,
        jwt: Option<VpnApiTime>,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self
            .inner
            .create_post_request(path, NO_PARAMS, json_body)?
            .bearer_auth(account.jwt(jwt).to_string());

        let request = match device {
            Some(device) => request.header(
                DEVICE_AUTHORIZATION_HEADER,
                format!("Bearer {}", device.jwt(jwt)),
            ),
            None => request,
        };
        let response = request.send().await?;
        nym_http_api_client::parse_response(response, false).await
    }

    async fn post_authorized<T, B>(
        &self,
        path: PathSegments<'_>,
        json_body: &B,
        account: &VpnAccount,
        device: Option<&Device>,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        match self
            .post_query::<T, B>(path, json_body, account, device, None)
            .await
        {
            Ok(response) => Ok(response),
            Err(err) => {
                if let HttpClientError::EndpointFailure { error, .. } = &err
                    && jwt_error(&error.to_string())
                {
                    tracing::warn!(
                        "Encountered possible JWT error: {error}. Retrying query with remote time"
                    );
                    if let Ok(Some(jwt)) = self.sync_with_remote_time().await.inspect_err(|err| {
                        tracing::error!("Failed to get remote time: {err}. Not retrying anymore")
                    }) {
                        // retry with remote vpn api time, and return that only if it succeeds,
                        // otherwise return the initial error
                        let res = self
                            .post_query(path, json_body, account, device, Some(jwt))
                            .await;
                        if res.is_ok() {
                            return res;
                        }
                    }
                }
                Err(err)
            }
        }
    }

    async fn delete_query<T>(
        &self,
        path: PathSegments<'_>,
        account: &VpnAccount,
        device: Option<&Device>,
        jwt: Option<VpnApiTime>,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        let request = self
            .inner
            .create_delete_request(path, NO_PARAMS)?
            .bearer_auth(account.jwt(jwt).to_string());

        let request = match device {
            Some(device) => request.header(
                DEVICE_AUTHORIZATION_HEADER,
                format!("Bearer {}", device.jwt(jwt)),
            ),
            None => request,
        };
        let response = request.send().await?;
        nym_http_api_client::parse_response(response, false).await
    }

    async fn delete_authorized<T>(
        &self,
        path: PathSegments<'_>,
        account: &VpnAccount,
        device: Option<&Device>,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        match self.delete_query::<T>(path, account, device, None).await {
            Ok(response) => Ok(response),
            Err(err) => {
                if let HttpClientError::EndpointFailure { error, .. } = &err
                    && jwt_error(&error.to_string())
                {
                    tracing::warn!(
                        "Encountered possible JWT error: {error}. Retrying query with remote time"
                    );
                    if let Ok(Some(jwt)) = self.sync_with_remote_time().await.inspect_err(|err| {
                        tracing::error!("Failed to get remote time: {err}. Not retring anymore")
                    }) {
                        // retry with remote vpn api time, and return that only if it succeeds,
                        // otherwise return the initial error
                        let res = self.delete_query(path, account, device, Some(jwt)).await;
                        if res.is_ok() {
                            return res;
                        }
                    }
                }
                Err(err)
            }
        }
    }

    async fn patch_query<T, B>(
        &self,
        path: PathSegments<'_>,
        json_body: &B,
        account: &VpnAccount,
        device: Option<&Device>,
        jwt: Option<VpnApiTime>,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self
            .inner
            .create_patch_request(path, NO_PARAMS, json_body)?
            .bearer_auth(account.jwt(jwt).to_string());

        let request = match device {
            Some(device) => request.header(
                DEVICE_AUTHORIZATION_HEADER,
                format!("Bearer {}", device.jwt(jwt)),
            ),
            None => request,
        };
        let response = request.send().await?;
        nym_http_api_client::parse_response(response, false).await
    }

    async fn patch_authorized<T, B>(
        &self,
        path: PathSegments<'_>,
        json_body: &B,
        account: &VpnAccount,
        device: Option<&Device>,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        match self
            .patch_query::<T, B>(path, json_body, account, device, None)
            .await
        {
            Ok(response) => Ok(response),
            Err(err) => {
                if let HttpClientError::EndpointFailure { error, .. } = &err
                    && jwt_error(&error.to_string())
                {
                    tracing::warn!(
                        "Encountered possible JWT error: {error}. Retrying query with remote time"
                    );
                    if let Ok(Some(jwt)) = self.sync_with_remote_time().await.inspect_err(|err| {
                        tracing::error!("Failed to get remote time: {err}. Not retring anymore")
                    }) {
                        // retry with remote vpn api time, and return that only if it succeeds,
                        // otherwise return the initial error
                        let res = self
                            .patch_query(path, json_body, account, device, Some(jwt))
                            .await;
                        if res.is_ok() {
                            return res;
                        }
                    }
                }
                Err(err)
            }
        }
    }

    // ACCOUNT

    pub async fn get_account(&self, account: &VpnAccount) -> Result<NymVpnAccountResponse> {
        self.get_authorized(
            &[routes::PUBLIC, routes::V1, routes::ACCOUNT, &account.id()],
            account,
            None,
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetAccount)
    }

    async fn post_account<B>(
        &self,
        platform_path: &str,
        body: &B,
    ) -> Result<NymVpnRegisterAccountResponse>
    where
        B: Serialize + ?Sized + Sync,
    {
        self.post_json_with_retry(
            &[routes::PUBLIC, routes::V1, routes::ACCOUNT, platform_path],
            NO_PARAMS,
            &body,
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::PostAccount)
    }

    async fn register_apple_account<B>(&self, body: &B) -> Result<NymVpnRegisterAccountResponse>
    where
        B: Serialize + ?Sized + Sync,
    {
        self.post_account(routes::APPLE, body).await
    }

    async fn register_android_account<B>(&self, body: &B) -> Result<NymVpnRegisterAccountResponse>
    where
        B: Serialize + ?Sized + Sync,
    {
        self.post_account(routes::ANDROID, body).await
    }

    pub async fn register_account(
        &self,
        account: &VpnAccount,
        platform: Platform,
    ) -> Result<NymVpnRegisterAccountResponse> {
        let account_addr = account.id().to_string();
        let pub_key = account.pub_key().to_string();
        let signature_base64 = account.signature_base64().to_string();
        match platform {
            Platform::Apple => {
                self.register_apple_account(&CreateAppleAccountRequestBody {
                    account_addr,
                    pub_key,
                    signature_base64,
                })
                .await
            }
            Platform::Android { purchase_token } => {
                self.register_android_account(&CreateAndroidAccountRequestBody {
                    account_addr,
                    pub_key,
                    signature_base64,
                    purchase_token,
                })
                .await
            }
        }
    }

    pub async fn get_health(&self) -> Result<NymVpnHealthResponse> {
        self.get_json_with_retry(&[routes::PUBLIC, routes::V1, routes::HEALTH], NO_PARAMS)
            .await
            .map_err(Box::new)
            .map_err(VpnApiClientError::GetHealth)
    }

    pub async fn get_wellknown_envs(&self) -> Result<crate::response::RegisteredNetworksResponse> {
        self.inner
            .get_json(
                &[
                    routes::PUBLIC,
                    routes::V1,
                    routes::WELLKNOWN,
                    routes::ENVS_FILE,
                ],
                NO_PARAMS,
            )
            .await
            .map_err(Box::new)
            .map_err(VpnApiClientError::GetWellknownEnvs)
    }

    pub async fn get_wellknown_discovery(
        &self,
        network: &str,
    ) -> Result<crate::response::NymWellknownDiscoveryItemResponse> {
        self.inner
            .get_json(
                &[
                    routes::PUBLIC,
                    routes::V1,
                    routes::WELLKNOWN,
                    network,
                    routes::DISCOVERY_FILE,
                ],
                NO_PARAMS,
            )
            .await
            .map_err(Box::new)
            .map_err(VpnApiClientError::GetWellknownDiscovery)
    }

    pub async fn get_account_summary(
        &self,
        account: &VpnAccount,
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetAccountSummary)
    }

    pub async fn get_account_summary_with_device(
        &self,
        account: &VpnAccount,
        device: &Device,
    ) -> Result<NymVpnAccountSummaryWithDeviceResponse> {
        self.get_authorized(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
                &device.identity_key().to_string(),
                routes::SUMMARY,
            ],
            account,
            Some(device),
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetAccountSummaryWithDevice)
    }

    // DEVICES

    pub async fn get_devices(&self, account: &VpnAccount) -> Result<NymVpnDevicesResponse> {
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetDevices)
    }

    pub async fn register_device(
        &self,
        account: &VpnAccount,
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::RegisterDevice)
    }

    pub async fn get_active_devices(&self, account: &VpnAccount) -> Result<NymVpnDevicesResponse> {
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetActiveDevices)
    }

    pub async fn get_device_by_id(
        &self,
        account: &VpnAccount,
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetDeviceById)
    }

    pub async fn update_device(
        &self,
        account: &VpnAccount,
        device: &Device,
        status: DeviceStatus,
    ) -> Result<NymVpnDevice> {
        let body = UpdateDeviceRequestBody {
            status: UpdateDeviceRequestStatus::from(status),
        };

        self.patch_authorized(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
                &device.identity_key().to_string(),
            ],
            &body,
            account,
            Some(device),
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::UpdateDevice)
    }

    // ZK-NYM

    pub async fn get_device_zk_nyms(
        &self,
        account: &VpnAccount,
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetDeviceZkNyms)
    }

    pub async fn request_zk_nym(
        &self,
        account: &VpnAccount,
        device: &Device,
        withdrawal_request: String,
        ecash_pubkey: String,
        expiration_date: String,
        ticketbook_type: String,
    ) -> Result<NymVpnZkNymPost> {
        tracing::debug!("Requesting zk-nym for type: {ticketbook_type}");
        let body = RequestZkNymRequestBody {
            withdrawal_request,
            ecash_pubkey,
            expiration_date,
            ticketbook_type,
        };
        tracing::debug!("Request body: {body:#?}");

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
        .map_err(Box::new)
        .map_err(VpnApiClientError::RequestZkNym)
    }

    pub async fn get_zk_nyms_available_for_download(
        &self,
        account: &VpnAccount,
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
                routes::AVAILABLE,
            ],
            account,
            Some(device),
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetDeviceZkNyms)
    }

    pub async fn get_zk_nym_by_id(
        &self,
        account: &VpnAccount,
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetZkNymById)
    }

    pub async fn confirm_zk_nym_download_by_id(
        &self,
        account: &VpnAccount,
        device: &Device,
        id: &str,
    ) -> Result<StatusOk> {
        self.delete_authorized(
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::ConfirmZkNymDownloadById)
    }

    // FREEPASS

    pub async fn get_free_passes(
        &self,
        account: &VpnAccount,
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetFreePasses)
    }

    pub async fn apply_freepass(
        &self,
        account: &VpnAccount,
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::ApplyFreepass)
    }

    // SUBSCRIPTIONS

    pub async fn get_subscriptions(
        &self,
        account: &VpnAccount,
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetSubscriptions)
    }

    pub async fn create_subscription(&self, account: &VpnAccount) -> Result<NymVpnSubscription> {
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::CreateSubscription)
    }

    pub async fn get_active_subscriptions(
        &self,
        account: &VpnAccount,
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetActiveSubscriptions)
    }

    pub async fn get_usage(&self, account: &VpnAccount) -> Result<NymVpnUsagesResponse> {
        self.get_authorized(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::USAGE,
            ],
            account,
            None,
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetUsage)
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetGateways)
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetVpnGateways)
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetVpnGatewayCountries)
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetGatewayCountries)
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetEntryGateways)
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetEntryGatewayCountries)
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetExitGateways)
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
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetExitGatewayCountries)
    }

    // DIRECTORY ZK-NYM

    pub async fn get_directory_zk_nyms_ticketbook_partial_verification_keys(
        &self,
    ) -> Result<PartialVerificationKeysResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::ZK_NYMS,
                routes::TICKETBOOK,
                routes::PARTIAL_VERIFICATION_KEYS,
            ],
            NO_PARAMS,
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetDirectoryZkNymsTicketbookPartialVerificationKeys)
    }

    pub async fn get_wellknown_current_env(&self) -> Result<NymWellknownDiscoveryItem> {
        tracing::debug!("Fetching nym vpn network details");
        self.inner
            .get_json(
                &[
                    routes::PUBLIC,
                    routes::V1,
                    routes::WELLKNOWN,
                    routes::CURRENT_ENV,
                ],
                NO_PARAMS,
            )
            .await
            .map_err(Box::new)
            .map_err(VpnApiClientError::GetVpnNetworkDetails)
    }
}

fn jwt_error(error: &str) -> bool {
    error.to_lowercase().contains("jwt")
}
