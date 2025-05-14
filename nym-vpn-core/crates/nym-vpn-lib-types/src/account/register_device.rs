// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Debug;

use super::VpnApiError;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum RegisterDeviceError {
    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("register device: {0}")]
    RegisterDeviceEndpointFailure(VpnApiError),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("no connectivity")]
    Offline,

    #[error("internal error: {0}")]
    Internal(String),
}

impl RegisterDeviceError {
    pub fn unexpected_response(message: impl Debug) -> Self {
        RegisterDeviceError::UnexpectedResponse(format!("{message:?}"))
    }

    pub fn internal(message: impl ToString) -> Self {
        RegisterDeviceError::Internal(message.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            RegisterDeviceError::NoAccountStored => self.to_string(),
            RegisterDeviceError::NoDeviceStored => self.to_string(),
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => failure.message(),
            RegisterDeviceError::UnexpectedResponse(message) => message.clone(),
            RegisterDeviceError::Offline => self.to_string(),
            RegisterDeviceError::Internal(_) => self.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => failure.message_id(),
            RegisterDeviceError::NoAccountStored
            | RegisterDeviceError::NoDeviceStored
            | RegisterDeviceError::UnexpectedResponse(_)
            | RegisterDeviceError::Offline
            | RegisterDeviceError::Internal(_) => None,
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        match self {
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => {
                failure.code_reference_id()
            }
            RegisterDeviceError::NoAccountStored
            | RegisterDeviceError::NoDeviceStored
            | RegisterDeviceError::UnexpectedResponse(_)
            | RegisterDeviceError::Offline
            | RegisterDeviceError::Internal(_) => None,
        }
    }
}
