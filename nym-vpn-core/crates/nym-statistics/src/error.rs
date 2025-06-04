// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StatsApiClient(#[from] nym_statistics_api_client::StatisticsApiClientError),

    #[error("storage error : {source}")]
    StatsStorage {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
