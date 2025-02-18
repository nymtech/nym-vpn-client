// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::VpnApiErrorResponse;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum StoreAccountError {
    #[error("storage: {0}")]
    Storage(String),

    #[error("vpn api endpoint failure: {0}")]
    GetAccountEndpointFailure(VpnApiErrorResponse),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}

impl StoreAccountError {
    pub fn message(&self) -> String {
        match self {
            StoreAccountError::Storage(message) => message.clone(),
            StoreAccountError::GetAccountEndpointFailure(failure) => failure.message.clone(),
            StoreAccountError::UnexpectedResponse(response) => response.clone(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            StoreAccountError::Storage(_) => None,
            StoreAccountError::GetAccountEndpointFailure(failure) => failure.message_id.clone(),
            StoreAccountError::UnexpectedResponse(_) => None,
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        match self {
            StoreAccountError::Storage(_) => None,
            StoreAccountError::GetAccountEndpointFailure(failure) => {
                failure.code_reference_id.clone()
            }
            StoreAccountError::UnexpectedResponse(_) => None,
        }
    }
}
