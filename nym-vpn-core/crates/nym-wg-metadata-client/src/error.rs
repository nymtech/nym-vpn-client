// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::HttpClientError;

#[derive(Debug, thiserror::Error)]
pub enum MetadataClientError {
    #[error(transparent)]
    HttpClientError(#[from] Box<HttpClientError>),

    #[error(transparent)]
    NymWireguardMetadataClient(#[from] nym_wireguard_private_metadata_shared::error::MetadataError),

    #[error("this version of metadata endpoint is not supported")]
    UnsupportedMetadataEndpointVersion,

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, MetadataClientError>;
