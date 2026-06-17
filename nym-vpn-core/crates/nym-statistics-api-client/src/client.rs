// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_http_api_client::{ApiClient, HttpClientError, NO_PARAMS, UserAgent};
use nym_statistics_common::report::vpn_client::{ActiveDeviceReport, VpnClientStatsReportV2};
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

use crate::{
    error::{Result, StatisticsApiClientError},
    routes,
};

// requests can unfortunately take a long time over the mixnet
pub(crate) const NYM_STATISTICS_API_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub struct StatisticsApiClient {
    inner: nym_http_api_client::Client,
    #[cfg(target_os = "ios")]
    base_url: Url,
    #[cfg(target_os = "ios")]
    user_agent: UserAgent,
}

impl StatisticsApiClient {
    pub fn new(base_url: Url, user_agent: UserAgent) -> Result<Self> {
        // What about domain fronting?  The discovery schema makes no provision for it.
        Ok(Self {
            #[cfg(target_os = "ios")]
            base_url: base_url.clone(),
            #[cfg(target_os = "ios")]
            user_agent: user_agent.clone(),
            inner: Self::make_http_client(
                base_url,
                user_agent,
                #[cfg(target_os = "ios")]
                None,
            )?,
        })
    }

    /// Create a copy of this client with its sockets bound to the given network interface.
    ///
    /// On iOS, traffic from the packet tunnel provider is excluded from the tunnel, so the
    /// socket must be explicitly bound to the tun interface for reports to be wrapped in the
    /// tunnel. Hostnames are resolved through the system resolver, which is served by the
    /// in-process DNS forwarder bound to the tunnel while it is up; the default DoH/DoT
    /// resolver would create unbound sockets of its own and leak outside of the tunnel.
    #[cfg(target_os = "ios")]
    pub fn with_bound_interface(&self, interface: Option<&str>) -> Result<Self> {
        let inner =
            Self::make_http_client(self.base_url.clone(), self.user_agent.clone(), interface)?;

        Ok(Self {
            base_url: self.base_url.clone(),
            user_agent: self.user_agent.clone(),
            inner,
        })
    }

    fn make_http_client(
        base_url: Url,
        user_agent: UserAgent,
        #[cfg(target_os = "ios")] bound_interface: Option<&str>,
    ) -> Result<nym_http_api_client::Client> {
        nym_http_api_client::Client::builder(base_url)
            .and_then(|builder| {
                let builder = builder
                    .with_user_agent(user_agent)
                    .with_timeout(NYM_STATISTICS_API_TIMEOUT);

                #[cfg(target_os = "ios")]
                let builder = if let Some(bound_interface) = bound_interface {
                    let reqwest_builder = nym_http_api_client::registry::default_builder()
                        .timeout(NYM_STATISTICS_API_TIMEOUT)
                        .interface(bound_interface);

                    // Enforce the use of system resolver which is set to in-process DNS forwarder that prevents leaks.
                    let reqwest_builder = reqwest_builder.no_hickory_dns();

                    builder.with_reqwest_builder(reqwest_builder)
                } else {
                    builder
                };

                builder.build()
            })
            .map_err(Box::new)
            .map_err(StatisticsApiClientError::VpnApiClientCreation)
    }

    async fn post_query<T, B>(
        &self,
        path: &str,
        json_body: &B,
    ) -> std::result::Result<T, HttpClientError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let request = self.inner.create_post_request(path, NO_PARAMS, json_body)?;
        let response = request.send().await?;

        // parse_response currently can't handle empty response without throwing an error because it will try to deserialize it anyway
        nym_http_api_client::parse_response(response, false).await
    }

    pub async fn post_session_report(&self, report: VpnClientStatsReportV2) -> Result<()> {
        self.post_query(routes::SESSION_REPORT_ROUTE, &report)
            .await
            .map_err(Box::new)
            .map_err(StatisticsApiClientError::ReportSending)
    }

    pub async fn post_active_device(&self, report: ActiveDeviceReport) -> Result<()> {
        self.post_query(routes::ACTIVE_DEVICE_ROUTE, &report)
            .await
            .map_err(Box::new)
            .map_err(StatisticsApiClientError::ReportSending)
    }
}
