// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::responses::UnexpectedError;

#[derive(Debug, thiserror::Error)]
pub enum VpnApiClientError {
    #[error(transparent)]
    HttpClientError(#[from] nym_http_api_client::HttpClientError<UnexpectedError>),
}

pub type Result<T> = std::result::Result<T, VpnApiClientError>;
