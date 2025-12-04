// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StatsApiClient(#[from] Box<nym_statistics_api_client::StatisticsApiClientError>),

    #[error("storage error : {0}")]
    StatsStorage(#[from] crate::storage::error::StatsStorageError),

    #[error("failed to initialzed storage")]
    NoStorage,

    #[error("Some reports couldn't be sent")]
    ReportSending,

    // Internal error that should not happen, channel related
    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    pub fn internal(message: impl ToString) -> Self {
        Error::Internal(message.to_string())
    }
}
