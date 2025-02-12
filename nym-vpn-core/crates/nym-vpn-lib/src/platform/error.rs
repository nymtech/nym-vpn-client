// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::AccountCommandError;

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq)]
pub enum VpnError {
    #[error("internal error:{details}")]
    InternalError { details: String },

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

    #[error("no device identity stored")]
    NoDeviceIdentity,

    #[error("vpn-api error: {details}")]
    VpnApi {
        #[from]
        details: super::uniffi_lib_types::VpnApiErrorResponse,
    },

    #[error("timeout connecting to nym-vpn-api")]
    VpnApiTimeout,

    #[error("failed to parse mnemonic with error: {details}")]
    InvalidMnemonic { details: String },

    #[error("invalid account storage path: {details}")]
    InvalidAccountStoragePath { details: String },

    #[error("failed to remove device from nym vpn api: {details}")]
    UnregisterDevice { details: String },

    #[error("failed to store account: {details}")]
    StoreAccount {
        #[from]
        details: super::uniffi_lib_types::StoreAccountError,
    },

    #[error("sync account failed: {details}")]
    SyncAccount {
        #[from]
        details: super::uniffi_lib_types::SyncAccountError,
    },
    #[error("sync device failed: {details}")]
    SyncDevice {
        #[from]
        details: super::uniffi_lib_types::SyncDeviceError,
    },

    #[error("device registration failed: {details}")]
    RegisterDevice {
        #[from]
        details: super::uniffi_lib_types::RegisterDeviceError,
    },

    #[error("failed to request zk nym")]
    RequestZkNym {
        #[from]
        details: super::uniffi_lib_types::RequestZkNymError,
    },

    #[error("when requesting zk nym, some were reported as failed")]
    RequestZkNymBundle {
        successes: Vec<super::uniffi_lib_types::RequestZkNymSuccess>,
        failed: Vec<super::uniffi_lib_types::RequestZkNymError>,
    },

    #[error("failed to forget account: {details}")]
    ForgetAccount {
        #[from]
        details: super::uniffi_lib_types::ForgetAccountError,
    },
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
            AccountCommandError::VpnApi(e) => Self::VpnApi { details: e.into() },
            AccountCommandError::NoAccountStored => Self::NoAccountStored,
            AccountCommandError::NoDeviceStored => Self::NoDeviceIdentity,
            AccountCommandError::StoreAccount(e) => Self::StoreAccount { details: e.into() },
            AccountCommandError::SyncAccount(e) => Self::SyncAccount { details: e.into() },
            AccountCommandError::SyncDevice(e) => Self::SyncDevice { details: e.into() },
            AccountCommandError::RegisterDevice(e) => Self::RegisterDevice { details: e.into() },
            AccountCommandError::RequestZkNym(e) => Self::RequestZkNym { details: e.into() },
            AccountCommandError::RequestZkNymBundle { successes, failed } => {
                Self::RequestZkNymBundle {
                    successes: successes.into_iter().map(|e| e.into()).collect(),
                    failed: failed.into_iter().map(|e| e.into()).collect(),
                }
            }
            AccountCommandError::ForgetAccount(e) => Self::ForgetAccount { details: e.into() },
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

impl From<nym_vpn_store::keys::persistence::OnDiskKeysError> for VpnError {
    fn from(value: nym_vpn_store::keys::persistence::OnDiskKeysError) -> Self {
        Self::Storage {
            details: value.to_string(),
        }
    }
}

impl From<nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorageError> for VpnError {
    fn from(value: nym_vpn_store::mnemonic::on_disk::OnDiskMnemonicStorageError) -> Self {
        Self::Storage {
            details: value.to_string(),
        }
    }
}
