// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::Serialize;

use crate::{config::StatisticsControllerConfig, error::Error};

#[derive(Clone, Debug)]
pub(crate) struct StatisticsControllerApiClient {
    inner: nym_statistics_api_client::StatisticsApiClient,
}

impl StatisticsControllerApiClient {
    pub fn new(config: &StatisticsControllerConfig) -> Result<Option<Self>, Error> {
        if let Some(url) = &config.stats_collector_url {
            let inner_api_client = nym_statistics_api_client::StatisticsApiClient::new(
                url.clone(),
                config.user_agent.clone(),
            )
            .map_err(Box::new)?;
            Ok(Some(StatisticsControllerApiClient::from(inner_api_client)))
        } else {
            Ok(None)
        }
    }
    pub async fn post_report(&self, report: impl Serialize) -> Result<(), Error> {
        self.inner.post_stats_report(report).await.map_err(Box::new)?;
        Ok(())
    }
}

impl From<nym_statistics_api_client::StatisticsApiClient> for StatisticsControllerApiClient {
    fn from(statistics_api_client: nym_statistics_api_client::StatisticsApiClient) -> Self {
        Self {
            inner: statistics_api_client,
        }
    }
}
