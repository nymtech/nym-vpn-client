// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{NodeIdentity, Recipient};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("received invalid response from gateway authenticator")]
    InvalidGatewayAuthResponse,

    #[error(transparent)]
    AuthenticatorClientError(#[from] nym_authenticator_client::Error),

    #[error(transparent)]
    WireguardTypesError(#[from] nym_wireguard_types::error::Error),

    #[error("failed to parse entry gateway socket addr: {0}")]
    FailedToParseEntryGatewaySocketAddr(#[source] std::net::AddrParseError),

    #[error("out of bandwidth with gateway: `{gateway_id}")]
    OutOfBandwidth {
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
    },
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
