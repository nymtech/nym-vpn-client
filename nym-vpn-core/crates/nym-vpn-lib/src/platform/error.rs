// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum VpnError {
    #[error("{inner}")]
    Configuration { inner: String },

    #[error("{inner}")]
    TunnelShutdown { inner: String },

    #[error("{inner}")]
    ApiClient { inner: String },

    #[error("{inner}")]
    GatewayDirectory { inner: String },

    #[error("{inner}")]
    InvalidState { inner: String },

    #[error("{inner}")]
    Credential { inner: String },
}

impl From<crate::Error> for VpnError {
    fn from(value: crate::Error) -> Self {
        Self::TunnelShutdown {
            inner: value.to_string(),
        }
    }
}

impl From<nym_gateway_directory::Error> for VpnError {
    fn from(value: nym_gateway_directory::Error) -> Self {
        Self::GatewayDirectory {
            inner: value.to_string(),
        }
    }
}

impl From<nym_vpn_api_client::VpnApiClientError> for VpnError {
    fn from(value: nym_vpn_api_client::VpnApiClientError) -> Self {
        Self::ApiClient {
            inner: value.to_string(),
        }
    }
}
