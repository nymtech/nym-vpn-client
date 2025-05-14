// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt::Debug, sync::Arc};

pub mod forget_account;
pub mod register_device;
pub mod request_zknym;
pub mod store_account;
pub mod sync_account;
pub mod sync_device;
pub mod ticketbooks;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AccountCommandError {
    // Internal error that should not happen
    #[error("internal error: {0}")]
    Internal(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("vpn-api error: {0}")]
    VpnApi(#[from] VpnApiError),

    #[error("unexpected vpn-api response: {0}")]
    UnexpectedVpnApiResponse(String),

    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("no connectivity")]
    Offline,

    //
    // --- Error cases for specific commands ---
    //
    #[error("failed to store account: {0}")]
    StoreAccount(#[from] store_account::StoreAccountError),

    #[error("failed to sync account state: {0}")]
    SyncAccount(#[from] sync_account::SyncAccountError),

    #[error("failed to sync device state: {0}")]
    SyncDevice(#[from] sync_device::SyncDeviceError),

    #[error("failed to register device: {0}")]
    RegisterDevice(#[from] register_device::RegisterDeviceError),

    #[error("failed to request zk nym: {0}")]
    RequestZkNym(#[from] request_zknym::RequestZkNymError),

    #[error("failed to request zk nym")]
    RequestZkNymBundle {
        successes: Vec<request_zknym::RequestZkNymSuccess>,
        failed: Vec<request_zknym::RequestZkNymError>,
    },

    #[error("failed to forget account: {0}")]
    ForgetAccount(#[from] forget_account::ForgetAccountError),
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

// Local alias for syntactic simplification
type RequestZkNymVec =
    Vec<Result<request_zknym::RequestZkNymSuccess, request_zknym::RequestZkNymError>>;

impl From<RequestZkNymVec> for AccountCommandError {
    fn from(summary: RequestZkNymVec) -> Self {
        let (successes, failed): (Vec<_>, Vec<_>) = summary.into_iter().partition(Result::is_ok);
        let successes = successes.into_iter().map(Result::unwrap).collect();
        let failed = failed.into_iter().map(Result::unwrap_err).collect();
        Self::RequestZkNymBundle { successes, failed }
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

impl TryFrom<nym_vpn_api_client::VpnApiClientError> for VpnApiError {
    type Error = nym_vpn_api_client::VpnApiClientError;

    fn try_from(err: nym_vpn_api_client::VpnApiClientError) -> Result<Self, Self::Error> {
        let err = match VpnApiErrorResponse::try_from(err) {
            Ok(err) => return Ok(Self::Response(err)),
            Err(err) => err,
        };

        if nym_vpn_api_client::response::error_is_reqwest_timeout(&err) {
            return Ok(Self::Timeout(Arc::new(err)));
        }

        match nym_vpn_api_client::response::extract_error_response_status_code(&err) {
            Some(code) => Ok(Self::StatusCode {
                code,
                source: Arc::new(err),
            }),
            None => Err(err),
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
impl TryFrom<nym_vpn_api_client::VpnApiClientError> for VpnApiErrorResponse {
    type Error = nym_vpn_api_client::VpnApiClientError;

    fn try_from(err: nym_vpn_api_client::VpnApiClientError) -> Result<Self, Self::Error> {
        nym_vpn_api_client::response::NymErrorResponse::try_from(err).map(|res| Self {
            message: res.message,
            message_id: res.message_id,
            code_reference_id: res.code_reference_id,
        })
    }
}
