// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::error::Error;
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

// todo: rename this type back to ErrorStateReason once support for renaming uniffi structs is released
//       see: https://github.com/mozilla/uniffi-rs/issues/2212
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum AccountControllerErrorStateReason {
    /// Error due to storage
    Storage { context: String, details: String },

    /// API Failure
    ApiFailure { context: String, details: String },

    // Let's limit that type to the minimum
    /// Internal
    Internal { context: String, details: String },

    // ==== User need to do something for these below ==== //
    /// Bandwidth Exceeded
    BandwidthExceeded { context: String },

    /// Account status is not "Active"
    AccountStatusNotActive { status: String },

    /// Inactive Subscription
    InactiveSubscription,

    /// Max device numbers reached
    MaxDeviceReached,

    /// Device time is off by too much, Zk-nyms use will fail
    DeviceTimeDesynced,
}

impl AccountControllerErrorStateReason {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::ApiFailure { .. } | Self::Internal { .. })
    }

    pub fn storage(
        context: &'static str,
        details: impl Into<String>,
    ) -> AccountControllerErrorStateReason {
        AccountControllerErrorStateReason::Storage {
            context: context.to_string(),
            details: details.into(),
        }
    }

    pub fn storage_err(
        context: &'static str,
        details: impl Error,
    ) -> AccountControllerErrorStateReason {
        Self::storage(context, details.to_string())
    }
}

impl std::fmt::Display for AccountControllerErrorStateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountControllerErrorStateReason::Storage { context, details } => {
                write!(f, "Storage error: {context} - {details} ")
            }
            AccountControllerErrorStateReason::ApiFailure { context, details } => {
                write!(f, "API failure: {context} - {details}")
            }
            AccountControllerErrorStateReason::Internal { context, details } => {
                write!(f, "Internal error: {context} - {details}")
            }
            AccountControllerErrorStateReason::BandwidthExceeded { context } => {
                write!(f, "Bandwidth exceeded: {context}")
            }
            AccountControllerErrorStateReason::AccountStatusNotActive { status } => {
                write!(f, "Account status not active: {status}")
            }
            AccountControllerErrorStateReason::InactiveSubscription => {
                write!(f, "Inactive subscription")
            }
            AccountControllerErrorStateReason::MaxDeviceReached => {
                write!(f, "Max device numbers reached")
            }
            AccountControllerErrorStateReason::DeviceTimeDesynced => {
                write!(f, "Device time is off by too much")
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Error))]
pub enum AccountControllerError {
    #[error("Account controller is offline")]
    Offline,

    #[error("Account controller has no account stored")]
    NoAccountStored,

    #[error("Internal error : {0}")]
    Internal(String),

    #[error("Account controller is in error state. Reason : {0}")]
    ErrorState(AccountControllerErrorStateReason),
}
