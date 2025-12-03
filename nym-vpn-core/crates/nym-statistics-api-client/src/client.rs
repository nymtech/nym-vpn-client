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
}

impl StatisticsApiClient {
    pub fn new(base_url: Url, user_agent: UserAgent) -> Result<Self> {
        // What about domain fronting?  The discovery schema makes no provision for it.
        nym_http_api_client::Client::builder(base_url.clone())
            .and_then(|builder| {
                builder
                    .with_user_agent(user_agent)
                    .with_timeout(NYM_STATISTICS_API_TIMEOUT)
                    .build()
            })
            .map(|c| Self { inner: c })
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
