// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub use nym_http_api_client::HttpClientError;

#[derive(Debug, thiserror::Error)]
pub enum StatisticsApiClientError {
    #[error("failed to create vpn api client")]
    VpnApiClientCreation(#[source] Box<HttpClientError>),

    #[error("failed to post statistics report : {0}")]
    ReportSending(#[source] Box<HttpClientError>),
}

pub type Result<T> = std::result::Result<T, StatisticsApiClientError>;
