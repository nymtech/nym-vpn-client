// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Debug;

use super::VpnApiError;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum RegisterAccountError {
    #[error("offline")]
    Offline,

    #[error("storage: {0}")]
    Storage(String),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("register account")]
    RegisterAccountEndpointFailure(VpnApiError),

    #[error("internal error: {0}")]
    Internal(String),
}

impl RegisterAccountError {
    pub fn internal(err: impl ToString) -> Self {
        RegisterAccountError::Internal(err.to_string())
    }

    pub fn storage(err: impl ToString) -> Self {
        RegisterAccountError::Storage(err.to_string())
    }

    pub fn unexpected_response(err: impl Debug) -> Self {
        RegisterAccountError::UnexpectedResponse(format!("{err:?}"))
    }

    pub fn message(&self) -> String {
        match self {
            RegisterAccountError::Offline => String::from("offline"),
            RegisterAccountError::Storage(message) => message.clone(),
            RegisterAccountError::UnexpectedResponse(response) => response.clone(),
            RegisterAccountError::RegisterAccountEndpointFailure(failure) => failure.message(),
            RegisterAccountError::Internal(message) => message.clone(),
        }
    }
}
