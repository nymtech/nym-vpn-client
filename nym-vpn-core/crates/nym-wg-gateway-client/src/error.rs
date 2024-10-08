// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{NodeIdentity, Recipient};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("received invalid response from gateway authenticator")]
    InvalidGatewayAuthResponse,

    #[error(transparent)]
    AuthenticatorClientError(#[from] nym_authenticator_client::Error),

    #[error("verification failed: {0}")]
    VerificationFailed(#[source] nym_authenticator_requests::Error),

    #[error("failed to parse entry gateway socket addr: {0}")]
    FailedToParseEntryGatewaySocketAddr(#[source] std::net::AddrParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorMessage {
    #[error("out of bandwidth for gateway: `{gateway_id}`")]
    OutOfBandwidth {
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
    },
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
