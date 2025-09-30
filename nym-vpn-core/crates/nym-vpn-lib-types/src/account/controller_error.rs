// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ErrorStateReason {
    /// Error due to storage
    Storage { context: String },

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

impl std::fmt::Display for ErrorStateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorStateReason::Storage { context } => write!(f, "Storage error: {context}"),
            ErrorStateReason::ApiFailure { context, details } => {
                write!(f, "API failure: {context} - {details}")
            }
            ErrorStateReason::Internal { context, details } => {
                write!(f, "Internal error: {context} - {details}")
            }
            ErrorStateReason::BandwidthExceeded { context } => {
                write!(f, "Bandwidth exceeded: {context}")
            }
            ErrorStateReason::AccountStatusNotActive { status } => {
                write!(f, "Account status not active: {status}")
            }
            ErrorStateReason::InactiveSubscription => write!(f, "Inactive subscription"),
            ErrorStateReason::MaxDeviceReached => write!(f, "Max device numbers reached"),
            ErrorStateReason::DeviceTimeDesynced => write!(f, "Device time is off by too much"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum AccountControllerError {
    #[error("Account controller is offline")]
    Offline,

    #[error("Account controller has no account stored")]
    NoAccountStored,

    #[error("Internal error : {0}")]
    Internal(String),

    #[error("Account controller is in error state. Reason : {0}")]
    ErrorState(ErrorStateReason),
}
