// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum FFIError {
    #[error("Invalid value passed in uniffi")]
    InvalidValueUniffi,

    #[error("Could not obtain a fd")]
    FdNotFound,

    #[error("VPN wasn't stopped properly")]
    VpnNotStopped,

    #[error("VPN wasn't started properly")]
    VpnNotStarted,

    #[cfg(target_os = "android")]
    #[error("Context was not initialised")]
    NoContext,

    #[error("{inner}")]
    LibError { inner: String },
}

impl From<crate::Error> for FFIError {
    fn from(value: crate::Error) -> Self {
        Self::LibError {
            inner: value.to_string(),
        }
    }
}
