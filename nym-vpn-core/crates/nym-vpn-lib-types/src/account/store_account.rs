// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Debug;

use super::VpnApiErrorResponse;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum StoreAccountError {
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("storage: {0}")]
    Storage(String),

    #[error("vpn api endpoint failure: {0}")]
    GetAccountEndpointFailure(VpnApiErrorResponse),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl StoreAccountError {
    pub fn internal(err: impl ToString) -> Self {
        StoreAccountError::Internal(err.to_string())
    }

    pub fn storage(err: impl ToString) -> Self {
        StoreAccountError::Storage(err.to_string())
    }

    pub fn unexpected_response(err: impl Debug) -> Self {
        StoreAccountError::UnexpectedResponse(format!("{err:?}"))
    }

    pub fn message(&self) -> String {
        match self {
            StoreAccountError::InvalidMnemonic(message) => message.clone(),
            StoreAccountError::Storage(message) => message.clone(),
            StoreAccountError::GetAccountEndpointFailure(failure) => failure.message.clone(),
            StoreAccountError::UnexpectedResponse(response) => response.clone(),
            StoreAccountError::Internal(message) => message.clone(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        if let StoreAccountError::GetAccountEndpointFailure(failure) = self {
            failure.message_id.clone()
        } else {
            None
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        if let StoreAccountError::GetAccountEndpointFailure(failure) = self {
            failure.code_reference_id.clone()
        } else {
            None
        }
    }
}
