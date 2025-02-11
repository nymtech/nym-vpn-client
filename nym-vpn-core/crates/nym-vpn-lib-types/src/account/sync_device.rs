// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::VpnApiErrorResponse;

#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum SyncDeviceError {
    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("vpn api endpoint failure: {0}")]
    SyncDeviceEndpointFailure(VpnApiErrorResponse),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl SyncDeviceError {
    pub fn unexpected_response(err: impl ToString) -> Self {
        SyncDeviceError::UnexpectedResponse(err.to_string())
    }

    pub fn internal(err: impl ToString) -> Self {
        SyncDeviceError::Internal(err.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            SyncDeviceError::NoAccountStored => self.to_string(),
            SyncDeviceError::NoDeviceStored => self.to_string(),
            SyncDeviceError::SyncDeviceEndpointFailure(failure) => failure.message.clone(),
            SyncDeviceError::UnexpectedResponse(response) => response.to_string(),
            SyncDeviceError::Internal(_) => self.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            SyncDeviceError::NoAccountStored => None,
            SyncDeviceError::NoDeviceStored => None,
            SyncDeviceError::SyncDeviceEndpointFailure(failure) => failure.message_id.clone(),
            SyncDeviceError::UnexpectedResponse(_) => None,
            SyncDeviceError::Internal(_) => None,
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        match self {
            SyncDeviceError::NoAccountStored => None,
            SyncDeviceError::NoDeviceStored => None,
            SyncDeviceError::SyncDeviceEndpointFailure(failure) => {
                failure.code_reference_id.clone()
            }
            SyncDeviceError::UnexpectedResponse(_) => None,
            SyncDeviceError::Internal(_) => None,
        }
    }
}
