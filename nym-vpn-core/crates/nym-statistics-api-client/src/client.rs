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
    /// Kept around to rebuild the client bound to the tunnel interface (iOS).
    #[cfg_attr(not(target_os = "ios"), allow(dead_code))]
    base_url: Url,
    #[cfg_attr(not(target_os = "ios"), allow(dead_code))]
    user_agent: UserAgent,
}

impl StatisticsApiClient {
    pub fn new(base_url: Url, user_agent: UserAgent) -> Result<Self> {
        // What about domain fronting?  The discovery schema makes no provision for it.
        nym_http_api_client::Client::builder(base_url.clone())
            .and_then(|builder| {
                builder
                    .with_user_agent(user_agent.clone())
                    .with_timeout(NYM_STATISTICS_API_TIMEOUT)
                    .build()
            })
            .map(|c| Self {
                inner: c,
                base_url,
                user_agent,
            })
            .map_err(Box::new)
            .map_err(StatisticsApiClientError::VpnApiClientCreation)
    }

    /// Create a copy of this client with its sockets bound to the given network interface.
    ///
    /// On iOS, traffic from the packet tunnel provider is excluded from the tunnel, so the
    /// socket must be explicitly bound to the tun interface for reports to be wrapped in the
    /// tunnel. Hostnames are resolved through the system resolver, which is served by the
    /// in-process DNS forwarder bound to the tunnel while it is up; the default DoH/DoT
    /// resolver would create unbound sockets of its own and leak outside of the tunnel.
    #[cfg(target_os = "ios")]
    pub fn with_bound_interface(&self, interface: &str) -> Result<Self> {
        let reqwest_builder = nym_http_api_client::registry::default_builder()
            .interface(interface)
            .timeout(NYM_STATISTICS_API_TIMEOUT);
        nym_http_api_client::Client::builder(self.base_url.clone())
            .and_then(|builder| {
                builder
                    .with_user_agent(self.user_agent.clone())
                    .with_timeout(NYM_STATISTICS_API_TIMEOUT)
                    .with_reqwest_builder(reqwest_builder)
                    .no_hickory_dns()
                    .build()
            })
            .map(|inner| Self {
                inner,
                base_url: self.base_url.clone(),
                user_agent: self.user_agent.clone(),
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
