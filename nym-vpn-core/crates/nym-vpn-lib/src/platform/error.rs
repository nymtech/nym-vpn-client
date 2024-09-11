// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq)]
pub enum VpnError {
    #[error("{details}")]
    InternalError { details: String },

    #[error("{details}")]
    NetworkConnectionError { details: String },

    #[error("{details}")]
    GatewayError { details: String },

    #[error("{details}")]
    InvalidCredential { details: String },

    #[error("Client is out of bandwidth")]
    OutOfBandwidth,

    #[error("{details}")]
    InvalidStateError { details: String },
}

impl From<crate::Error> for VpnError {
    fn from(value: crate::Error) -> Self {
        Self::InternalError {
            details: value.to_string(),
        }
    }
}

impl From<nym_gateway_directory::Error> for VpnError {
    fn from(value: nym_gateway_directory::Error) -> Self {
        Self::NetworkConnectionError {
            details: value.to_string(),
        }
    }
}

impl From<nym_vpn_api_client::VpnApiClientError> for VpnError {
    fn from(value: nym_vpn_api_client::VpnApiClientError) -> Self {
        Self::NetworkConnectionError {
            details: value.to_string(),
        }
    }
}
