// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, net::SocketAddr, time::Duration};

use nym_http_api_client::{ApiClient, HttpClientError, NO_PARAMS, PathSegments, UserAgent};
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

use crate::{
    error::{Result, StatisticsApiClientError},
    routes,
};

// requests can unfortunately take a long time over the mixnet
pub(crate) const NYM_STATISTICS_API_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug)]
pub struct StatisticsApiClient {
    inner: nym_http_api_client::Client,
}

impl StatisticsApiClient {
    pub fn new(base_url: Url, user_agent: UserAgent) -> Result<Self> {
        Self::new_with_resolver_overrides(base_url, user_agent, None)
    }

    pub fn new_with_resolver_overrides(
        base_url: Url,
        user_agent: UserAgent,
        static_addresses: Option<&[SocketAddr]>,
    ) -> Result<Self> {
        nym_http_api_client::Client::builder(base_url.clone())
            .map(|builder| {
                let mut builder = builder
                    .with_user_agent(user_agent)
                    .with_timeout(NYM_STATISTICS_API_TIMEOUT);

                if let Some(domain) = base_url.domain() {
                    match static_addresses {
                        Some(static_addresses) if !static_addresses.is_empty() => {
                            tracing::info!(
                                "Enabling DNS resolver overrides: {:?}", static_addresses
                            );
                            builder = builder.resolve_to_addrs(domain, static_addresses);
                        }
                        Some(_) => {
                            tracing::warn!(
                                "Not enabling DNS resolver overrides because static addresses are empty"
                            );
                        }
                        None => {
                            tracing::info!(
                                "Not enabling DNS resolver overrides because static addresses are not set"
                            );
                        }
                    }
                } else {
                    tracing::info!(
                        "Not enabling DNS resolver overrides because domain is not present in base URL"
                    );
                }

                builder
            })
            .and_then(|builder| builder.build())
            .map(|c| Self { inner: c })
            .map_err(StatisticsApiClientError::FailedToCreateVpnApiClient)
    }

    pub fn swap_inner_client(&mut self, client: StatisticsApiClient) {
        self.inner = client.inner;
    }

    pub fn current_url(&self) -> &Url {
        self.inner.current_url()
    }

    async fn post_query<T, B, E>(
        &self,
        path: PathSegments<'_>,
        json_body: &B,
    ) -> std::result::Result<T, HttpClientError<E>>
    where
        T: DeserializeOwned,
        B: Serialize,
        E: fmt::Display + DeserializeOwned,
    {
        let request = self.inner.create_post_request(path, NO_PARAMS, json_body);

        let response = request.send().await?;

        //SW parse_response currently can't handle empty response without throwing an error because it will try to deserialize it anyway
        nym_http_api_client::parse_response(response, false).await
    }

    pub async fn post_stats_report<B>(&self, body: B) -> Result<()>
    where
        B: Serialize,
    {
        self.post_query(&[routes::V1, routes::STATS, routes::REPORT], &body)
            .await
            .map_err(StatisticsApiClientError::FailedToPostReport)
    }
}
