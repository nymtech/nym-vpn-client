// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt::Debug, sync::Arc};

pub mod controller_error;
pub mod controller_event;
pub mod controller_state;
pub mod request_zknym;
pub mod ticketbooks;

#[derive(Clone, Debug)]
pub struct RegisterAccountResponse {
    pub account_token: String,
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AccountCommandError {
    // Internal error that should not happen
    #[error("internal error: {0}")]
    Internal(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("vpn-api error")]
    VpnApi(#[from] VpnApiError),

    #[error("unexpected vpn-api response: {0}")]
    UnexpectedVpnApiResponse(String),

    #[error("failed to connect to nyxd instance: {0}")]
    NyxdConnectionFailure(String),

    #[error("failed to resolve query to a nyxd instance: {0}")]
    NyxdQueryFailure(String),

    #[error("account doesn't exist on chain")]
    AccountDoesntExistOnChain,

    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("an account is already stored")]
    ExistingAccount,

    #[error("no connectivity")]
    Offline,

    #[error("account is not set in decentralised mode")]
    AccountNotDecentralised,

    #[error("account is set in decentralised mode")]
    AccountDecentralised,

    #[error("account does not have sufficient funds")]
    InsufficientFunds,

    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),
}

impl AccountCommandError {
    pub fn internal(message: impl ToString) -> Self {
        AccountCommandError::Internal(message.to_string())
    }

    pub fn storage(message: impl ToString) -> Self {
        AccountCommandError::Storage(message.to_string())
    }

    pub fn unexpected_response(message: impl Debug) -> Self {
        AccountCommandError::UnexpectedVpnApiResponse(format!("{message:?}"))
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnApiError {
    #[error("timeout")]
    Timeout(#[source] Arc<dyn std::error::Error + Send + Sync>),

    #[error("status code: {code}")]
    StatusCode {
        code: u16,
        source: Arc<dyn std::error::Error + Send + Sync>,
    },

    #[error(transparent)]
    Response(#[from] VpnApiErrorResponse),
}

// We want to keep the source error for logging, while at the same time it needs to be PartialEq
// and Eq. This is a workaround to make it work.

impl PartialEq for VpnApiError {
    fn eq(&self, other: &Self) -> bool {
        use VpnApiError::*;
        match (self, other) {
            (Timeout(a), Timeout(b)) => a.to_string() == b.to_string(),
            (
                StatusCode {
                    code: a,
                    source: a_source,
                },
                StatusCode {
                    code: b,
                    source: b_source,
                },
            ) => a == b && a_source.to_string() == b_source.to_string(),
            (Response(err), Response(other_err)) => err == other_err,
            _ => false,
        }
    }
}

impl Eq for VpnApiError {}

impl VpnApiError {
    pub fn message(&self) -> String {
        match self {
            VpnApiError::Response(err) => err.message.clone(),
            VpnApiError::StatusCode { .. } => self.to_string(),
            VpnApiError::Timeout(_) => self.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        if let VpnApiError::Response(err) = self {
            err.message_id.clone()
        } else {
            None
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        if let VpnApiError::Response(err) = self {
            err.code_reference_id.clone()
        } else {
            None
        }
    }
}

// That should disappear when reworking those errors
#[cfg(feature = "nym-type-conversions")]
impl TryFrom<nym_vpn_api_client::error::VpnApiClientError> for VpnApiError {
    type Error = nym_vpn_api_client::error::VpnApiClientError;

    fn try_from(err: nym_vpn_api_client::error::VpnApiClientError) -> Result<Self, Self::Error> {
        let err = match VpnApiErrorResponse::try_from(err) {
            Ok(err) => return Ok(Self::Response(err)),
            Err(err) => err,
        };

        if err
            .http_client_error()
            .is_some_and(nym_vpn_api_client::error::HttpClientError::is_timeout)
        {
            return Ok(Self::Timeout(Arc::new(err)));
        }

        match err
            .http_client_error()
            .and_then(nym_vpn_api_client::error::HttpClientError::status_code)
        {
            Some(code) => Ok(Self::StatusCode {
                code: code.into(),
                source: Arc::new(err),
            }),
            None => Err(err),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::error::VpnApiClientError> for AccountCommandError {
    fn from(err: nym_vpn_api_client::error::VpnApiClientError) -> Self {
        use nym_vpn_api_client::response::NymErrorResponse;
        // TODO: Another example of losing information about the original error cause
        match NymErrorResponse::try_from(err) {
            Ok(err) => AccountCommandError::VpnApi(VpnApiError::Response(err.into())),
            Err(e) => AccountCommandError::Internal(e.to_string()),
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("{message}, message_id: {message_id:?}, code_reference_id: {code_reference_id:?}")]
pub struct VpnApiErrorResponse {
    pub message: String,
    pub message_id: Option<String>,
    pub code_reference_id: Option<String>,
}

#[cfg(feature = "nym-type-conversions")]
impl TryFrom<nym_vpn_api_client::error::VpnApiClientError> for VpnApiErrorResponse {
    type Error = nym_vpn_api_client::error::VpnApiClientError;

    fn try_from(err: nym_vpn_api_client::error::VpnApiClientError) -> Result<Self, Self::Error> {
        Ok(VpnApiErrorResponse::from(
            nym_vpn_api_client::response::NymErrorResponse::try_from(err)?,
        ))
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::NymErrorResponse> for VpnApiErrorResponse {
    fn from(err: nym_vpn_api_client::response::NymErrorResponse) -> Self {
        Self {
            message: err.message,
            message_id: err.message_id,
            code_reference_id: err.code_reference_id,
        }
    }
}
