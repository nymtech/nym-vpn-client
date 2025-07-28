// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, Clone, Eq, PartialEq, strum_macros::Display)]
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
