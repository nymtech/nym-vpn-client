// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::AccountCommandError;

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq)]
pub enum VpnError {
    #[error("{details}")]
    InternalError { details: String },

    #[error("{details}")]
    NetworkConnectionError { details: String },

    #[error("{details}")]
    InvalidStateError { details: String },

    #[error("no account stored")]
    NoAccountStored,

    #[error("attempting to access an account that is not registered")]
    AccountNotRegistered,

    #[error("no device identity stored")]
    NoDeviceIdentity,

    #[error("timeout connecting to nym-vpn-api")]
    VpnApiTimeout,

    #[error("failed to parse mnemonic with error: {details}")]
    InvalidMnemonic { details: String },

    #[error("invalid account storage path: {details}")]
    InvalidAccountStoragePath { details: String },

    #[error("failed to remove device from nym vpn api: {details}")]
    UnregisterDevice { details: String },

    #[error("failed to store account: {0}")]
    StoreAccount(super::uniffi_lib_types::StoreAccountError),

    #[error("sync account failed: {0}")]
    SyncAccount(super::uniffi_lib_types::SyncAccountError),

    #[error("sync device failed: {0}")]
    SyncDevice(super::uniffi_lib_types::SyncDeviceError),

    #[error("device registration failed: {0}")]
    RegisterDevice(super::uniffi_lib_types::RegisterDeviceError),

    #[error("failed to request zk nym")]
    RequestZkNym(super::uniffi_lib_types::RequestZkNymError),

    #[error("when requesting zk nym, some were reported as failed")]
    RequestZkNymBundle {
        successes: Vec<super::uniffi_lib_types::RequestZkNymSuccess>,
        failed: Vec<super::uniffi_lib_types::RequestZkNymError>,
    },

    #[error("failed to forget account: {0}")]
    ForgetAccount(super::uniffi_lib_types::ForgetAccountError),
}

impl From<AccountCommandError> for VpnError {
    fn from(value: AccountCommandError) -> Self {
        match value {
            AccountCommandError::General(err) => Self::InternalError { details: err },
            AccountCommandError::Internal(err) => Self::InternalError { details: err },
            AccountCommandError::NoAccountStored => Self::NoAccountStored,
            AccountCommandError::NoDeviceStored => Self::NoDeviceIdentity,
            AccountCommandError::StoreAccount(e) => Self::StoreAccount(e.into()),
            AccountCommandError::SyncAccount(e) => Self::SyncAccount(e.into()),
            AccountCommandError::SyncDevice(e) => Self::SyncDevice(e.into()),
            AccountCommandError::RegisterDevice(e) => Self::RegisterDevice(e.into()),
            AccountCommandError::RequestZkNym(e) => Self::RequestZkNym(e.into()),
            AccountCommandError::RequestZkNymBundle { successes, failed } => {
                Self::RequestZkNymBundle {
                    successes: successes.into_iter().map(|e| e.into()).collect(),
                    failed: failed.into_iter().map(|e| e.into()).collect(),
                }
            }
            AccountCommandError::ForgetAccount(e) => Self::ForgetAccount(e.into()),
        }
    }
}

impl From<crate::Error> for VpnError {
    fn from(value: crate::Error) -> Self {
        Self::InternalError {
            details: value.to_string(),
        }
    }
}

impl From<nym_gateway_directory::Error> for VpnError {
    fn from(value: nym_gateway_directory::Error) -> Self {
        Self::NetworkConnectionError {
            details: value.to_string(),
        }
    }
}

impl From<nym_vpn_api_client::VpnApiClientError> for VpnError {
    fn from(value: nym_vpn_api_client::VpnApiClientError) -> Self {
        Self::NetworkConnectionError {
            details: value.to_string(),
        }
    }
}
