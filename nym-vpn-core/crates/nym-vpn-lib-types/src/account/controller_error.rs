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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum AccountControllerErrorStateReason {
    /// Error due to storage
    #[cfg_attr(feature = "cbor", n(0))]
    Storage {
        #[cfg_attr(feature = "cbor", n(0))]
        context: String,
        #[cfg_attr(feature = "cbor", n(1))]
        details: String,
    },

    /// API Failure
    #[cfg_attr(feature = "cbor", n(1))]
    ApiFailure {
        #[cfg_attr(feature = "cbor", n(0))]
        context: String,
        #[cfg_attr(feature = "cbor", n(1))]
        details: String,
    },

    // Let's limit that type to the minimum
    /// Internal
    #[cfg_attr(feature = "cbor", n(2))]
    Internal {
        #[cfg_attr(feature = "cbor", n(0))]
        context: String,
        #[cfg_attr(feature = "cbor", n(1))]
        details: String,
    },

    // ==== User need to do something for these below ==== //
    /// Bandwidth Exceeded
    #[cfg_attr(feature = "cbor", n(3))]
    BandwidthExceeded {
        #[cfg_attr(feature = "cbor", n(0))]
        context: String,
    },

    /// Account status is not "Active"
    #[cfg_attr(feature = "cbor", n(4))]
    AccountStatusNotActive {
        #[cfg_attr(feature = "cbor", n(0))]
        status: String,
    },

    /// Inactive Subscription
    #[cfg_attr(feature = "cbor", n(5))]
    InactiveSubscription,

    /// Max device numbers reached
    #[cfg_attr(feature = "cbor", n(6))]
    MaxDeviceReached,
}

impl AccountControllerErrorStateReason {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::ApiFailure { .. })
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
