// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use backon::Retryable;
use nym_credential_proxy_requests::api::v1::ticketbook::models::{
    AggregatedCoinIndicesSignaturesResponse, AggregatedExpirationDateSignaturesResponse,
    MasterVerificationKeyResponse, PartialVerificationKeysResponse,
};
use nym_http_api_client::{
    ApiClient, Client, HttpClientError, NO_PARAMS, Params, PathSegments, Url, UserAgent,
    reqwest::header::{DATE, HeaderMap},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use time::{
    Date, OffsetDateTime,
    format_description::{BorrowedFormatItem, well_known::Rfc2822},
};

use crate::{
    api_urls_to_urls,
    error::{Result, VpnApiClientError},
    fronted_http_client,
    request::{
        AccountKind, ApplyFreepassRequestBody, CreateAndroidAccountRequestBody,
        CreateAppleAccountRequestBody, CreateSubscriptionKind, CreateSubscriptionRequestBody,
        LinkAccountRequestBody, RegisterDeviceRequestBody, RequestZkNymRequestBody,
        UpdateDeviceRequestBody, UpdateDeviceRequestStatus,
    },
    response::{
        NymDirectoryGatewayCountriesResponse, NymDirectoryGatewaysResponse,
        NymUserGeoIpLocationResponse, NymVpnAccountResponse, NymVpnAccountSummaryResponse,
        NymVpnAccountSummaryWithDeviceResponse, NymVpnCanonicalAccountIdentityResponse,
        NymVpnDevice, NymVpnDevicesResponse, NymVpnHealthResponse, NymVpnRegisterAccountResponse,
        NymVpnSubscription, NymVpnSubscriptionResponse, NymVpnSubscriptionsResponse,
        NymVpnUsagesResponse, NymVpnZkNym, NymVpnZkNymPost, NymVpnZkNymResponse,
        NymWellknownDiscoveryItem, StatusOk,
    },
    routes,
    skew_manager::{RemoteTimeProvider, SkewManager},
    types::{
        Device, DeviceStatus, GatewayMinPerformance, GatewayType, Platform, VpnAccount, VpnApiTime,
    },
};

const RFC_3339_DATE: &[BorrowedFormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]");

pub(crate) const DEVICE_AUTHORIZATION_HEADER: &str = "x-device-authorization";

// GET requests can unfortunately take a long time over the mixnet
pub(crate) const NYM_VPN_API_TIMEOUT: Duration = Duration::from_secs(30);

/// Whether a request that failed with a JWT error should be retried after
/// synchronizing with the remote time.
#[derive(Clone, Copy, Debug)]
enum RemoteTimeRetry {
    /// Synchronization succeeded: retry the request, using the remote time
    /// override, or the local time when `None` (clocks are aligned).
    Retry(Option<VpnApiTime>),
    /// The remote time could not be determined: keep the original error.
    GiveUp,
}

#[derive(Clone, Debug)]
pub struct VpnApiClient {
    inner: Client,
    skew_manager: SkewManager,
}

impl AsRef<Client> for VpnApiClient {
    fn as_ref(&self) -> &Client {
        &self.inner
    }
}

impl AsMut<Client> for VpnApiClient {
    fn as_mut(&mut self) -> &mut Client {
        &mut self.inner
    }
}

impl VpnApiClient {
    /// Builds a `RemoteTimeProvider` that queries this VPN API's health endpoint, over its own
    /// dedicated HTTP client. Useful for constructing a `SkewManager` before any `VpnApiClient`
    /// exists, so it can be shared (injected via `new`/`from_network`) rather than owned solely
    /// by the client.
    pub fn health_endpoint_time_provider(
        urls: Vec<Url>,
        user_agent: Option<UserAgent>,
    ) -> Result<impl RemoteTimeProvider + Send + Sync + 'static> {
        let inner = fronted_http_client(urls, user_agent, Some(NYM_VPN_API_TIMEOUT))?;
        Ok(VpnApiRemoteTimeProvider::new(inner))
    }

    pub fn new(urls: Vec<Url>, user_agent: Option<UserAgent>) -> Result<Self> {
        let inner =
            fronted_http_client(urls.clone(), user_agent.clone(), Some(NYM_VPN_API_TIMEOUT))?;

        let skew_manager = SkewManager::new(VpnApiRemoteTimeProvider::new(inner.clone()));

        Ok(Self {
            inner,
            skew_manager,
        })
    }

    pub async fn from_network(
        network: &nym_network_defaults::NymNetworkDetails,
        user_agent: Option<UserAgent>,
    ) -> Result<Self> {
        #[allow(deprecated)]
        let api_urls = network.nym_vpn_api_urls.as_ref().ok_or_else(|| {
            let err: HttpClientError = HttpClientError::GenericRequestFailure(
                "No Nym VPN API URLs configured in network details".to_string(),
            );
            VpnApiClientError::CreateVpnApiClient(Box::new(err))
        })?;

        let urls = api_urls_to_urls(api_urls)?;

        let inner =
            fronted_http_client(urls.clone(), user_agent.clone(), Some(NYM_VPN_API_TIMEOUT))?;

        let skew_manager = SkewManager::new(VpnApiRemoteTimeProvider::new(inner.clone()));

        Ok(Self {
            inner,
            skew_manager,
        })
    }

    /// Replaces this client's skew manager with an externally-shared one (e.g. built via
    /// [`Self::health_endpoint_time_provider`]), so both observe and update the same cache
    /// instead of the client tracking its own, unshared skew state.
    pub fn with_skew_manager(mut self, skew_manager: SkewManager) -> Self {
        self.skew_manager = skew_manager;
        self
    }

    pub fn api_client(&self) -> &impl ApiClient {
        &self.inner
    }

    pub fn current_url(&self) -> &url::Url {
        self.inner.current_url().as_ref()
    }

    pub async fn get_remote_time(&self) -> Result<VpnApiTime> {
        self.skew_manager.get_remote_time().await
    }

    async fn device_time(&self) -> OffsetDateTime {
        self.skew_manager.device_time()
    }

    async fn remote_time_for_retry(
        &self,
        headers: &HeaderMap,
        time_before: OffsetDateTime,
        time_after: OffsetDateTime,
    ) -> RemoteTimeRetry {
        if let Some(remote_timestamp) = parse_date_header(headers) {
            match self
                .skew_manager
                .sync_with_response_timestamp(time_before, remote_timestamp, time_after)
                .await
            {
                Ok(jwt) => return RemoteTimeRetry::Retry(jwt),
                Err(err) => {
                    tracing::warn!(
                        "Failed to derive remote time from Date header: {err}. Falling back to fetching remote time"
                    );
                }
            }
        } else {
            tracing::debug!(
                "Response has no usable Date header, falling back to fetching remote time"
            );
        }

        match self.skew_manager.sync_with_remote_time().await {
            Ok(jwt) => RemoteTimeRetry::Retry(jwt),
            Err(err) => {
                tracing::error!("Failed to get remote time: {err}. Not retrying anymore");
                RemoteTimeRetry::GiveUp
            }
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
        let jwt = self
            .skew_manager
            .current_remote_time()
            .await
            .unwrap_or_else(|err| {
                tracing::debug!(
                    error = %err,
                    "Failed to determine cached remote time"
                );
                None
            });

        let time_before = self.device_time().await;
        match self.get_query::<T>(path, account, device, jwt).await {
            Ok(response) => Ok(response),
            Err(err) => {
                let time_after = self.device_time().await;
                if let HttpClientError::EndpointFailure { error, headers, .. } = &err
                    && jwt_error(&error.to_string())
                {
                    tracing::warn!(
                        "Encountered possible JWT error: {error}. Retrying query with remote time"
                    );
                    if let RemoteTimeRetry::Retry(jwt) = self
                        .remote_time_for_retry(headers, time_before, time_after)
                        .await
                    {
                        // retry with the freshly synchronized time (remote override, or local
                        // time when the clocks are aligned), and return the result only if it
                        // succeeds, otherwise return the initial error
                        let res = self.get_query(path, account, device, jwt).await;
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
                        url: Box::new(url),
                        status,
                        headers: Box::new(headers),
                        error: response_text,
                    })
                }
            }
            Err(err) => Err(HttpClientError::RequestFailure {
                url: Box::new(url),
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
        let jwt = self
            .skew_manager
            .current_remote_time()
            .await
            .unwrap_or_else(|err| {
                tracing::debug!(
                    error = %err,
                    "Failed to determine cached remote time"
                );
                None
            });

        let time_before = self.device_time().await;
        match self
            .post_query::<T, B>(path, json_body, account, device, jwt)
            .await
        {
            Ok(response) => Ok(response),
            Err(err) => {
                let time_after = self.device_time().await;
                if let HttpClientError::EndpointFailure { error, headers, .. } = &err
                    && jwt_error(&error.to_string())
                {
                    tracing::warn!(
                        "Encountered possible JWT error: {error}. Retrying query with remote time"
                    );
                    if let RemoteTimeRetry::Retry(jwt) = self
                        .remote_time_for_retry(headers, time_before, time_after)
                        .await
                    {
                        // retry with the freshly synchronized time (remote override, or local
                        // time when the clocks are aligned), and return the result only if it
                        // succeeds, otherwise return the initial error
                        let res = self.post_query(path, json_body, account, device, jwt).await;
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
        let jwt = self
            .skew_manager
            .current_remote_time()
            .await
            .unwrap_or_else(|err| {
                tracing::debug!(
                    error = %err,
                    "Failed to determine cached remote time"
                );
                None
            });

        let time_before = self.device_time().await;
        match self.delete_query::<T>(path, account, device, jwt).await {
            Ok(response) => Ok(response),
            Err(err) => {
                let time_after = self.device_time().await;
                if let HttpClientError::EndpointFailure { error, headers, .. } = &err
                    && jwt_error(&error.to_string())
                {
                    tracing::warn!(
                        "Encountered possible JWT error: {error}. Retrying query with remote time"
                    );
                    if let RemoteTimeRetry::Retry(jwt) = self
                        .remote_time_for_retry(headers, time_before, time_after)
                        .await
                    {
                        // retry with the freshly synchronized time (remote override, or local
                        // time when the clocks are aligned), and return the result only if it
                        // succeeds, otherwise return the initial error
                        let res = self.delete_query(path, account, device, jwt).await;
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
        let jwt = self
            .skew_manager
            .current_remote_time()
            .await
            .unwrap_or_else(|err| {
                tracing::debug!(
                    error = %err,
                    "Failed to determine cached remote time"
                );
                None
            });

        let time_before = self.device_time().await;
        match self
            .patch_query::<T, B>(path, json_body, account, device, jwt)
            .await
        {
            Ok(response) => Ok(response),
            Err(err) => {
                let time_after = self.device_time().await;
                if let HttpClientError::EndpointFailure { error, headers, .. } = &err
                    && jwt_error(&error.to_string())
                {
                    tracing::warn!(
                        "Encountered possible JWT error: {error}. Retrying query with remote time"
                    );
                    if let RemoteTimeRetry::Retry(jwt) = self
                        .remote_time_for_retry(headers, time_before, time_after)
                        .await
                    {
                        // retry with the freshly synchronized time (remote override, or local
                        // time when the clocks are aligned), and return the result only if it
                        // succeeds, otherwise return the initial error
                        let res = self
                            .patch_query(path, json_body, account, device, jwt)
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
        let kind = if account.mode().is_privy() {
            AccountKind::PrivySecp256k1
        } else {
            AccountKind::UserGeneratedSecp256k1
        };
        let pub_key = account.pub_key().to_string();
        let signature_base64 = account.signature_base64().to_string();
        match platform {
            Platform::Apple => {
                self.register_apple_account(&CreateAppleAccountRequestBody {
                    account_addr,
                    pub_key,
                    signature_base64,
                    kind,
                })
                .await
            }
            Platform::Android { purchase_token } => {
                self.register_android_account(&CreateAndroidAccountRequestBody {
                    account_addr,
                    pub_key,
                    signature_base64,
                    purchase_token,
                    kind,
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

    pub async fn link_account(
        &self,
        account: &VpnAccount,
        linked_account: &VpnAccount,
        label: &str,
    ) -> Result<StatusOk> {
        let pubkey = linked_account.pub_key().to_string();

        let signature_json = format!(
            r#"{{"canonical_account_addr":"{}","public_key_base58":"{}"}}"#,
            account.id(),
            linked_account.pub_key()
        );
        let signature = linked_account
            .sign(&signature_json)
            .map_err(Box::new)
            .map_err(VpnApiClientError::AccountError)?;

        let request = LinkAccountRequestBody {
            pubkey,
            signature,
            kind: "privy_secp256k1".to_string(),
            label: label.to_string(),
        };

        self.post_authorized(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::AUTH_METHOD,
            ],
            &request,
            account,
            None,
        )
        .await
        .inspect_err(|e| tracing::error!("Failed to link account: {e:?}"))
        .map_err(Box::new)
        .map_err(VpnApiClientError::LinkPrivyAccount)
    }

    pub async fn get_canonical_account_identity(
        &self,
        account: &VpnAccount,
    ) -> Result<NymVpnCanonicalAccountIdentityResponse> {
        self.get_authorized(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::CANONICAL,
            ],
            account,
            None,
        )
        .await
        .inspect_err(|e| tracing::error!("Failed to get canonical account identity: {e:?}"))
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetCanonicalAccountIdentity)
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

    pub async fn delete_device(
        &self,
        account: &VpnAccount,
        device_identity_key: &str,
    ) -> Result<serde_json::Value> {
        // Device removal is a PATCH to `delete_me`, not an HTTP DELETE
        let body = UpdateDeviceRequestBody {
            status: UpdateDeviceRequestStatus::from(DeviceStatus::DeleteMe),
        };
        self.patch_authorized(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::ACCOUNT,
                &account.id(),
                routes::DEVICE,
                device_identity_key,
            ],
            &body,
            account,
            None,
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::DeleteDevice)
    }

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

    pub async fn get_directory_zk_nyms_ticketbook_master_verification_key(
        &self,
        epoch_id: u64,
    ) -> Result<MasterVerificationKeyResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::ZK_NYMS,
                routes::TICKETBOOK,
                routes::MASTER_VERIFICATION_KEY,
            ],
            &[(routes::EPOCH_ID, epoch_id.to_string())],
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetDirectoryZkNymsTicketbookMasterVerificationKey)
    }

    pub async fn get_directory_zk_nyms_ticketbook_aggregated_coin_indices_signatures(
        &self,
        epoch_id: u64,
    ) -> Result<AggregatedCoinIndicesSignaturesResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::ZK_NYMS,
                routes::TICKETBOOK,
                routes::AGGREGATED_COIN_INDICES_SIGNATURES,
            ],
            &[(routes::EPOCH_ID, epoch_id.to_string())],
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetDirectoryZkNymsTicketbookAggregatedCoinIndicesSignatures)
    }

    pub async fn get_directory_zk_nyms_ticketbook_aggregated_expiration_date_signatures(
        &self,
        epoch_id: u64,
        expiration_date: Date,
    ) -> Result<AggregatedExpirationDateSignaturesResponse> {
        self.get_json_with_retry(
            &[
                routes::PUBLIC,
                routes::V1,
                routes::DIRECTORY,
                routes::ZK_NYMS,
                routes::TICKETBOOK,
                routes::AGGREGATED_EXPIRATION_DATE_SIGNATURES,
            ],
            &[
                (routes::EPOCH_ID, epoch_id.to_string()),
                (
                    routes::EXPIRATION_DATE,
                    expiration_date.format(&RFC_3339_DATE).unwrap(),
                ),
            ],
        )
        .await
        .map_err(Box::new)
        .map_err(VpnApiClientError::GetDirectoryZkNymsTicketbookAggregatedExpirationDateSignatures)
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

    pub async fn get_geo_ip(&self) -> Result<NymUserGeoIpLocationResponse> {
        tracing::debug!("Fetching user geolocation for determining gateway proximity");
        self.inner
            .get_json(
                &[routes::PUBLIC, routes::V1, routes::GEO, routes::IP],
                NO_PARAMS,
            )
            .await
            .map_err(Box::new)
            .map_err(VpnApiClientError::GetGeoIp)
    }
}

fn jwt_error(error: &str) -> bool {
    error.to_lowercase().contains("jwt")
}

// Extracts the remote timestamp from the `Date` header of a response. The header is formatted
// as an RFC 7231 IMF-fixdate (e.g. "Sun, 06 Nov 1994 08:49:37 GMT"), which is a valid RFC 2822
// date.
fn parse_date_header(headers: &HeaderMap) -> Option<OffsetDateTime> {
    let date = headers.get(DATE)?.to_str().ok()?;
    OffsetDateTime::parse(date, &Rfc2822)
        .inspect_err(|err| tracing::debug!("Failed to parse Date header '{date}': {err}"))
        .ok()
}

#[derive(Debug)]
struct VpnApiRemoteTimeProvider {
    inner: Client,
}

impl VpnApiRemoteTimeProvider {
    pub fn new(inner: Client) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl RemoteTimeProvider for VpnApiRemoteTimeProvider {
    async fn request_remote_time(&self) -> Result<OffsetDateTime> {
        let res: NymVpnHealthResponse = self
            .inner
            .get_json(&[routes::PUBLIC, routes::V1, routes::HEALTH], NO_PARAMS)
            .await
            .map_err(Box::new)
            .map_err(VpnApiClientError::GetHealth)?;

        Ok(res.timestamp_utc)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use nym_http_api_client::reqwest::header::HeaderValue;

    use super::*;
    use crate::skew_manager::DeviceTimeProvider;

    // Thu, 16 Jul 2026 12:00:00 UTC
    fn device_time() -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(1_784_203_200).unwrap()
    }

    #[derive(Debug)]
    struct FixedDeviceTimeProvider;

    impl DeviceTimeProvider for FixedDeviceTimeProvider {
        fn device_time(&self) -> OffsetDateTime {
            device_time()
        }
    }

    #[derive(Debug)]
    struct CountingRemoteTimeProvider {
        calls: Arc<AtomicUsize>,
        remote_time: OffsetDateTime,
    }

    #[async_trait::async_trait]
    impl RemoteTimeProvider for CountingRemoteTimeProvider {
        async fn request_remote_time(&self) -> Result<OffsetDateTime> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.remote_time)
        }
    }

    // Returns a client that never makes real network requests: the device clock is fixed and
    // remote time requests are counted and answered with `remote_time`.
    fn test_client(remote_time: OffsetDateTime) -> (VpnApiClient, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let remote_time_provider = CountingRemoteTimeProvider {
            calls: calls.clone(),
            remote_time,
        };
        let skew_manager =
            SkewManager::new_for_testing(FixedDeviceTimeProvider, remote_time_provider);
        let inner = fronted_http_client(
            vec![Url::new("https://nymvpn.test/api/", None).unwrap()],
            None,
            Some(NYM_VPN_API_TIMEOUT),
        )
        .unwrap();
        (
            VpnApiClient {
                inner,
                skew_manager,
            },
            calls,
        )
    }

    fn headers_with_date(date: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(DATE, HeaderValue::from_str(date).unwrap());
        headers
    }

    #[test]
    fn parse_date_header_accepts_imf_fixdate() {
        let headers = headers_with_date("Sun, 06 Nov 1994 08:49:37 GMT");
        let parsed = parse_date_header(&headers).expect("valid IMF-fixdate");
        assert_eq!(parsed.unix_timestamp(), 784_111_777);
    }

    #[test]
    fn parse_date_header_rejects_garbage() {
        assert!(parse_date_header(&headers_with_date("not a date")).is_none());
        assert!(parse_date_header(&HeaderMap::new()).is_none());
    }

    #[tokio::test]
    async fn date_header_skew_avoids_remote_time_request() {
        let (client, remote_calls) = test_client(device_time());
        // Remote time is 2 hours behind the device time
        let headers = headers_with_date("Thu, 16 Jul 2026 10:00:00 GMT");

        let RemoteTimeRetry::Retry(Some(jwt)) = client
            .remote_time_for_retry(&headers, device_time(), device_time())
            .await
        else {
            panic!("skew is significant, expected retry with remote time override");
        };

        assert_eq!(remote_calls.load(Ordering::SeqCst), 0);
        assert_eq!(jwt.local_time_ahead_skew().whole_hours(), 2);

        // The derived skew is cached and reused without any network request
        let cached = client
            .skew_manager
            .current_remote_time()
            .await
            .unwrap()
            .expect("cached skew is significant");
        assert_eq!(remote_calls.load(Ordering::SeqCst), 0);
        assert_eq!(cached.local_time_ahead_skew().whole_hours(), 2);
    }

    #[tokio::test]
    async fn date_header_negligible_skew_retries_with_local_time() {
        let (client, remote_calls) = test_client(device_time());
        // Remote time is only 30 seconds ahead of the device time: below the threshold
        let headers = headers_with_date("Thu, 16 Jul 2026 12:00:30 GMT");

        let retry = client
            .remote_time_for_retry(&headers, device_time(), device_time())
            .await;

        assert!(matches!(retry, RemoteTimeRetry::Retry(None)));
        assert_eq!(remote_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn aligned_date_header_corrects_stale_cached_skew_and_retries() {
        let (client, remote_calls) = test_client(device_time());

        // Seed the cache with a stale skew: the device clock used to be 2 hours ahead
        client
            .skew_manager
            .sync_with_response_timestamp(
                device_time(),
                device_time() - Duration::from_hours(2),
                device_time(),
            )
            .await
            .unwrap()
            .expect("seeded skew is significant");

        // The failing request's Date header shows the clocks are now aligned
        let headers = headers_with_date("Thu, 16 Jul 2026 12:00:00 GMT");
        let retry = client
            .remote_time_for_retry(&headers, device_time(), device_time())
            .await;

        // Synchronization succeeded, so the request must still be retried, now
        // with local time since the clocks are aligned
        assert!(matches!(retry, RemoteTimeRetry::Retry(None)));
        assert_eq!(remote_calls.load(Ordering::SeqCst), 0);

        // The stale cached skew was replaced: no override and no network request
        let cached = client.skew_manager.current_remote_time().await.unwrap();
        assert!(cached.is_none());
        assert_eq!(remote_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn missing_date_header_falls_back_to_remote_time_request() {
        let (client, remote_calls) = test_client(device_time() - Duration::from_hours(2));

        let RemoteTimeRetry::Retry(Some(jwt)) = client
            .remote_time_for_retry(&HeaderMap::new(), device_time(), device_time())
            .await
        else {
            panic!("skew is significant, expected retry with remote time override");
        };

        assert_eq!(remote_calls.load(Ordering::SeqCst), 1);
        assert_eq!(jwt.local_time_ahead_skew().whole_hours(), 2);
    }

    #[tokio::test]
    async fn malformed_date_header_falls_back_to_remote_time_request() {
        let (client, remote_calls) = test_client(device_time() - Duration::from_hours(2));
        let headers = headers_with_date("not a date");

        let RemoteTimeRetry::Retry(Some(jwt)) = client
            .remote_time_for_retry(&headers, device_time(), device_time())
            .await
        else {
            panic!("skew is significant, expected retry with remote time override");
        };

        assert_eq!(remote_calls.load(Ordering::SeqCst), 1);
        assert_eq!(jwt.local_time_ahead_skew().whole_hours(), 2);
    }

    #[tokio::test]
    async fn untrustworthy_request_time_falls_back_to_remote_time_request() {
        let (client, remote_calls) = test_client(device_time() - Duration::from_hours(2));
        let headers = headers_with_date("Thu, 16 Jul 2026 10:00:00 GMT");

        // The request appears to have finished before it started, so the Date header
        // cannot be trusted
        let RemoteTimeRetry::Retry(Some(jwt)) = client
            .remote_time_for_retry(
                &headers,
                device_time(),
                device_time() - Duration::from_secs(1),
            )
            .await
        else {
            panic!("skew is significant, expected retry with remote time override");
        };

        assert_eq!(remote_calls.load(Ordering::SeqCst), 1);
        assert_eq!(jwt.local_time_ahead_skew().whole_hours(), 2);
    }
}
