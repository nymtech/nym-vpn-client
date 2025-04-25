// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_credentials_interface::TicketType;
use nym_gateway_directory::{NodeIdentity, Recipient};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Received invalid response from gateway authenticator")]
    InvalidGatewayAuthResponse,

    #[error("Unknown authenticator version number")]
    UnsupportedAuthenticatorVersion,

    #[error(transparent)]
    AuthenticatorClientError(#[from] nym_authenticator_client::Error),

    #[error("Error that should stop auto retrying")]
    NoRetry {
        #[source]
        source: nym_authenticator_client::Error,
    },

    #[error("Verification failure")]
    VerificationFailed(#[source] nym_authenticator_requests::Error),

    #[error("Failed to parse entry gateway socket addr")]
    FailedToParseEntryGatewaySocketAddr(#[source] std::net::AddrParseError),

    #[error("Failed to get {ticketbook_type} ticket")]
    GetTicket {
        ticketbook_type: TicketType,
        #[source]
        source: nym_bandwidth_controller::error::BandwidthControllerError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorMessage {
    #[error("Out of bandwidth for gateway: {gateway_id}")]
    OutOfBandwidth {
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
    },
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
