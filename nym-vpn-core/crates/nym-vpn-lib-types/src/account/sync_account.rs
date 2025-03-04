// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::VpnApiErrorResponse;

#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum SyncAccountError {
    #[error("no account stored")]
    NoAccountStored,

    #[error(transparent)]
    SyncAccountEndpointFailure(VpnApiErrorResponse),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl SyncAccountError {
    pub fn unexpected_response(err: impl ToString) -> Self {
        SyncAccountError::UnexpectedResponse(err.to_string())
    }

    pub fn internal(err: impl ToString) -> Self {
        SyncAccountError::Internal(err.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            SyncAccountError::NoAccountStored => self.to_string(),
            SyncAccountError::SyncAccountEndpointFailure(failure) => failure.message.clone(),
            SyncAccountError::UnexpectedResponse(response) => response.to_string(),
            SyncAccountError::Internal(_) => self.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            SyncAccountError::NoAccountStored => None,
            SyncAccountError::SyncAccountEndpointFailure(failure) => failure.message_id.clone(),
            SyncAccountError::UnexpectedResponse(_) => None,
            SyncAccountError::Internal(_) => None,
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        match self {
            SyncAccountError::NoAccountStored => None,
            SyncAccountError::SyncAccountEndpointFailure(failure) => {
                failure.code_reference_id.clone()
            }
            SyncAccountError::UnexpectedResponse(_) => None,
            SyncAccountError::Internal(_) => None,
        }
    }
}
