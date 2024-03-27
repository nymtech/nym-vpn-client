// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use super::ClientState;

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum FFIError {
    #[error("Invalid value passed in uniffi")]
    InvalidValueUniffi,

    #[error("Could not obtain a fd")]
    FdNotFound,

    #[error("Incorrect state. We are {current:?} and should be {expected:?}")]
    IncorrectState {
        current: ClientState,
        expected: ClientState,
    },

    #[error("{inner}")]
    LibError { inner: String },

    #[error("{inner}")]
    GatewayDirectoryError { inner: String },
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
