// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StatsApiClient(#[from] Box<nym_statistics_api_client::StatisticsApiClientError>),

    #[error("storage error : {0}")]
    StatsStorage(#[from] crate::storage::error::StatsStorageError),
}
