// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub use nym_http_api_client::HttpClientError;

#[derive(Debug, thiserror::Error)]
pub enum StatisticsApiClientError {
    #[error("failed tp create vpn api client")]
    FailedToCreateVpnApiClient(#[source] HttpClientError),

    #[error("failed to post statistics report : {0}")]
    FailedToPostReport(#[source] HttpClientError),
}

pub type Result<T> = std::result::Result<T, StatisticsApiClientError>;
