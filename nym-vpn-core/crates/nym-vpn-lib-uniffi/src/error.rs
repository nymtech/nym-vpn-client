// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::HttpClientError;
use nym_vpn_lib::tunnel_state_machine;
use nym_vpn_lib_types::{AccountCommandError, AccountControllerError};

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq)]
pub enum VpnError {
    #[error("internal error:{details}")]
    InternalError { details: String },

    #[error("initialization error: {details}")]
    Initialization { details: String },

    #[error("failed to initialize logs: {details}")]
    InitLogs { details: String },

    #[error("storage error: {details}")]
    Storage { details: String },

    #[error("network error: {details}")]
    NetworkConnectionError { details: String },

    #[error("API usage error: {details}")]
    InvalidStateError { details: String },

    #[error("no account stored")]
    NoAccountStored,

    #[error("attempting to access an account that is not registered")]
    AccountNotRegistered,

    #[error("failed to register account: {details}")]
    FailedAccountRegistration { details: String },

    #[error("no device identity stored")]
    NoDeviceIdentity,

    #[error("vpn-api error: {details}")]
    VpnApi {
        #[from]
        details: nym_vpn_lib_types::VpnApiError,
    },

    #[error("unexpected response from nym-vpn-api: {details}")]
    UnexpectedVpnApiResponse { details: String },

    #[error("timeout connecting to nym-vpn-api")]
    VpnApiTimeout,

    #[error("failed to parse mnemonic with error: {details}")]
    InvalidMnemonic { details: String },

    #[error("failed to parse secret with error: {details}")]
    InvalidSecret { details: String },

    #[error("failed to remove device from nym vpn api: {details}")]
    UnregisterDevice { details: String },

    #[error("an account is already stored")]
    ExistingAccount,

    #[error("account controller error")]
    AccountControllerError { details: String },

    #[error("http client error: {0}")]
    HttpClient(String),

    #[error("account doesn't exist on chain")]
    AccountDoesntExistOnChain,

    #[error("account is not set in decentralised mode")]
    AccountNotDecentralised,

    #[error("account is set in decentralised mode")]
    AccountDecentralised,

    #[error("account does not have sufficient funds")]
    InsufficientFunds,

    #[error("failed to obtain zk-nym: {details}")]
    ZkNymAcquisitionFailure { details: String },

    #[error("failed to connect to nyxd instance: {details}")]
    NyxdConnectionFailure { details: String },

    #[error("failed to resolve query to a nyxd instance: {details}")]
    NyxdQueryFailure { details: String },

    #[error("deeplink error: {details}")]
    DeeplinkError { details: String },

    #[error("failed to fetch environment: {details}")]
    FetchEnvironment { details: String },

    #[error("failed to link privy account: {details}")]
    LinkPrivyAccount { details: String },
}

impl From<HttpClientError> for VpnError {
    fn from(value: HttpClientError) -> Self {
        Self::HttpClient(value.to_string())
    }
}

impl VpnError {
    pub fn internal(details: impl ToString) -> Self {
        Self::InternalError {
            details: details.to_string(),
        }
    }
}

impl From<AccountCommandError> for VpnError {
    fn from(value: AccountCommandError) -> Self {
        match value {
            AccountCommandError::Internal(err) => Self::InternalError { details: err },
            AccountCommandError::Storage(err) => Self::Storage { details: err },
            AccountCommandError::VpnApi(e) => Self::VpnApi { details: e },
            AccountCommandError::UnexpectedVpnApiResponse(e) => Self::InternalError { details: e },
            AccountCommandError::NoAccountStored => Self::NoAccountStored,
            AccountCommandError::NoDeviceStored => Self::NoDeviceIdentity,
            AccountCommandError::Offline => Self::NetworkConnectionError {
                details: "Unable to proceed with command since we are offline".to_owned(),
            },
            AccountCommandError::ExistingAccount => Self::ExistingAccount,
            AccountCommandError::InvalidMnemonic(details) => Self::InvalidMnemonic { details },
            AccountCommandError::InvalidSecret(details) => Self::InvalidSecret { details },
            AccountCommandError::NyxdConnectionFailure(details) => {
                Self::NyxdConnectionFailure { details }
            }
            AccountCommandError::NyxdQueryFailure(details) => Self::NyxdQueryFailure { details },
            AccountCommandError::AccountDoesntExistOnChain => Self::AccountDoesntExistOnChain,
            AccountCommandError::AccountNotDecentralised => Self::AccountNotDecentralised,
            AccountCommandError::AccountDecentralised => Self::AccountDecentralised,
            AccountCommandError::InsufficientFunds => Self::InsufficientFunds,
            AccountCommandError::ZkNymAcquisitionFailure(details) => {
                Self::ZkNymAcquisitionFailure { details }
            }
            AccountCommandError::DeeplinkError(message) => Self::DeeplinkError { details: message },
        }
    }
}

impl From<tunnel_state_machine::Error> for VpnError {
    fn from(value: tunnel_state_machine::Error) -> Self {
        Self::InternalError {
            details: value.to_string(),
        }
    }
}

impl From<nym_vpn_store::keys::device::OnDiskKeysError> for VpnError {
    fn from(value: nym_vpn_store::keys::device::OnDiskKeysError) -> Self {
        Self::Storage {
            details: value.to_string(),
        }
    }
}

impl From<nym_vpn_store::account::on_disk::OnDiskMnemonicStorageError> for VpnError {
    fn from(value: nym_vpn_store::account::on_disk::OnDiskMnemonicStorageError) -> Self {
        Self::Storage {
            details: value.to_string(),
        }
    }
}

impl From<AccountControllerError> for VpnError {
    fn from(value: AccountControllerError) -> Self {
        Self::Storage {
            details: value.to_string(),
        }
    }
}
