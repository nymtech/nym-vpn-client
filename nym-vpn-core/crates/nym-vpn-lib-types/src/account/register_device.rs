// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::VpnApiErrorResponse;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum RegisterDeviceError {
    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("failed to register device: {0}")]
    RegisterDeviceEndpointFailure(VpnApiErrorResponse),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl RegisterDeviceError {
    pub fn unexpected_response(message: impl ToString) -> Self {
        RegisterDeviceError::UnexpectedResponse(message.to_string())
    }

    pub fn internal(message: impl ToString) -> Self {
        RegisterDeviceError::Internal(message.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            RegisterDeviceError::NoAccountStored => self.to_string(),
            RegisterDeviceError::NoDeviceStored => self.to_string(),
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => failure.message.clone(),
            RegisterDeviceError::UnexpectedResponse(message) => message.clone(),
            RegisterDeviceError::Internal(_) => self.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            RegisterDeviceError::NoAccountStored => None,
            RegisterDeviceError::NoDeviceStored => None,
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => {
                failure.message_id.clone()
            }
            RegisterDeviceError::UnexpectedResponse(_) => None,
            RegisterDeviceError::Internal(_) => None,
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        match self {
            RegisterDeviceError::NoAccountStored => None,
            RegisterDeviceError::NoDeviceStored => None,
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => {
                failure.code_reference_id.clone()
            }
            RegisterDeviceError::UnexpectedResponse(_) => None,
            RegisterDeviceError::Internal(_) => None,
        }
    }
}
