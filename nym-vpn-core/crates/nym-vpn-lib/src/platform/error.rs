// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum FFIError {
    #[error("Invalid value passed in uniffi")]
    InvalidValueUniffi,

    #[error("Invalid credential passed in uniffi")]
    InvalidCredential,

    #[error("{inner}")]
    VpnApiClientError { inner: String },

    #[error("{inner}")]
    LibError { inner: String },

    #[error("{inner}")]
    GatewayDirectoryError { inner: String },

    #[error("Invalid path")]
    InvalidPath,

    #[error("VPN wasn't stopped properly")]
    VpnNotStopped,

    #[error("VPN wasn't started properly")]
    VpnNotStarted,

    #[error("VPN already running")]
    VpnAlreadyRunning,
}

impl From<crate::Error> for FFIError {
    fn from(value: crate::Error) -> Self {
        Self::LibError {
            inner: value.to_string(),
        }
    }
}

impl From<nym_gateway_directory::Error> for FFIError {
    fn from(value: nym_gateway_directory::Error) -> Self {
        Self::GatewayDirectoryError {
            inner: value.to_string(),
        }
    }
}

impl From<nym_vpn_api_client::VpnApiClientError> for FFIError {
    fn from(value: nym_vpn_api_client::VpnApiClientError) -> Self {
        Self::VpnApiClientError {
            inner: value.to_string(),
        }
    }
}
