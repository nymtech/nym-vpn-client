// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Debug;

use super::VpnApiError;

#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum SyncDeviceError {
    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("sync device: {0}")]
    SyncDeviceEndpointFailure(#[from] VpnApiError),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("no connectivity")]
    Offline,

    #[error("internal error: {0}")]
    Internal(String),
}

impl SyncDeviceError {
    pub fn unexpected_response(err: impl Debug) -> Self {
        SyncDeviceError::UnexpectedResponse(format!("{err:?}"))
    }

    pub fn internal(err: impl ToString) -> Self {
        SyncDeviceError::Internal(err.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            SyncDeviceError::NoAccountStored => self.to_string(),
            SyncDeviceError::NoDeviceStored => self.to_string(),
            SyncDeviceError::SyncDeviceEndpointFailure(failure) => failure.message(),
            SyncDeviceError::UnexpectedResponse(response) => response.to_string(),
            SyncDeviceError::Offline => self.to_string(),
            SyncDeviceError::Internal(_) => self.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            SyncDeviceError::SyncDeviceEndpointFailure(failure) => failure.message_id(),
            SyncDeviceError::NoAccountStored
            | SyncDeviceError::NoDeviceStored
            | SyncDeviceError::UnexpectedResponse(_)
            | SyncDeviceError::Offline
            | SyncDeviceError::Internal(_) => None,
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        match self {
            SyncDeviceError::SyncDeviceEndpointFailure(failure) => failure.code_reference_id(),
            SyncDeviceError::NoAccountStored
            | SyncDeviceError::NoDeviceStored
            | SyncDeviceError::UnexpectedResponse(_)
            | SyncDeviceError::Offline
            | SyncDeviceError::Internal(_) => None,
        }
    }
}
