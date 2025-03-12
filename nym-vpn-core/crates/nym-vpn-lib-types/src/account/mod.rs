// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod forget_account;
pub mod register_device;
pub mod request_zknym;
pub mod store_account;
pub mod sync_account;
pub mod sync_device;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AccountCommandError {
    // Internal error that should not happen
    #[error("internal error: {0}")]
    Internal(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("vpn api error: {0}")]
    VpnApi(#[from] VpnApiErrorResponse),

    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

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
