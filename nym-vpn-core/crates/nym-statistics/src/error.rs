// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Statistics API client error : {0}")]
    StatsApiClient(#[from] nym_statistics_api_client::StatisticsApiClientError),
}
