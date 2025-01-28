// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::VpnApiErrorResponse;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum RegisterDeviceError {
    #[error("failed to register device: {0}")]
    RegisterDeviceEndpointFailure(VpnApiErrorResponse),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}

impl RegisterDeviceError {
    pub fn unexpected_response(message: impl ToString) -> Self {
        RegisterDeviceError::UnexpectedResponse(message.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => failure.message.clone(),
            RegisterDeviceError::UnexpectedResponse(message) => message.clone(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => {
                failure.message_id.clone()
            }
            RegisterDeviceError::UnexpectedResponse(_) => None,
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        match self {
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => {
                failure.code_reference_id.clone()
            }
            RegisterDeviceError::UnexpectedResponse(_) => None,
        }
    }
}
