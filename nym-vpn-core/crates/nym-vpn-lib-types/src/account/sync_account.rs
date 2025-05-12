// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error, fmt::Debug};

use nym_vpn_api_client::VpnApiClientError;

use super::VpnApiErrorResponse;

#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum SyncAccountError {
    #[error("no account stored")]
    NoAccountStored,

    #[error(transparent)]
    SyncAccountEndpointFailure(VpnApiErrorResponse),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("no connectivity")]
    Offline,

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<VpnApiClientError> for SyncAccountError {
    fn from(err: VpnApiClientError) -> Self {
        let err = match VpnApiErrorResponse::try_from(err) {
            Ok(vpn_api_error_response) => {
                return SyncAccountError::SyncAccountEndpointFailure(vpn_api_error_response);
            }
            Err(e) => e
        };

        if let Some(source) = err.source() {
            if let Some(source) = source.source() {
                // source.downcast_ref
            }
        }

        todo!();
    }
}

impl SyncAccountError {
    pub fn unexpected_response(err: impl Debug + std::fmt::Display + std::error::Error) -> Self {
        println!("Unexpected response: {err}");
        println!("Unexpected response: {err:?}");
        println!("Unexpected response: {}", err.source().unwrap());
        println!("Unexpected response: {:?}", err.source().unwrap());
        println!(
            "Unexpected response: {}",
            err.source().unwrap().source().unwrap()
        );
        println!(
            "Unexpected response: {:?}",
            err.source().unwrap().source().unwrap()
        );
        println!(
            "Unexpected response: {}",
            err.source().unwrap().source().unwrap().source().unwrap()
        );
        println!(
            "Unexpected response: {:?}",
            err.source().unwrap().source().unwrap().source().unwrap()
        );
        SyncAccountError::UnexpectedResponse(format!("{err:?}"))
    }

    pub fn internal(err: impl ToString) -> Self {
        SyncAccountError::Internal(err.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            SyncAccountError::NoAccountStored => self.to_string(),
            SyncAccountError::SyncAccountEndpointFailure(failure) => failure.message.clone(),
            SyncAccountError::UnexpectedResponse(response) => response.to_string(),
            SyncAccountError::Offline => self.to_string(),
            SyncAccountError::Internal(_) => self.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            SyncAccountError::SyncAccountEndpointFailure(failure) => failure.message_id.clone(),
            SyncAccountError::NoAccountStored
            | SyncAccountError::UnexpectedResponse(_)
            | SyncAccountError::Offline
            | SyncAccountError::Internal(_) => None,
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        match self {
            SyncAccountError::SyncAccountEndpointFailure(failure) => {
                failure.code_reference_id.clone()
            }
            SyncAccountError::NoAccountStored
            | SyncAccountError::UnexpectedResponse(_)
            | SyncAccountError::Offline
            | SyncAccountError::Internal(_) => None,
        }
    }
}
