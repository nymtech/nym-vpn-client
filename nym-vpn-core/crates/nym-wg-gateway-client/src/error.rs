// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum WgGatewayClientError {
    #[error("received invalid response from gateway authenticator")]
    InvalidGatewayAuthResponse,

    #[error(transparent)]
    AuthenticatorClientError(#[from] nym_authenticator_client::Error),

    #[error(transparent)]
    WireguardTypesError(#[from] nym_wireguard_types::error::Error),

    #[error("failed to parse entry gateway socket addr: {0}")]
    FailedToParseEntryGatewaySocketAddr(#[source] std::net::AddrParseError),

    #[error("out of bandwidth")]
    OutOfBandwidth,
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, WgGatewayClientError>;
