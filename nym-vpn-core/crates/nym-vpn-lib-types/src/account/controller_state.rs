// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use crate::AccountControllerErrorStateReason;

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
pub enum AccountControllerState {
    /// We don't have network
    Offline,

    /// Figuring out account state
    Syncing,

    /// Not logged in with a mnemonic
    LoggedOut,

    /// Logged in, registered device
    ReadyToConnect,

    /// Logged in, operating independently of VPN API
    Decentralised,

    /// Logged in, subscription is pending (e.g. cash payment still processing)
    PendingSubscription,

    /// Logged in, error during sync, can't proceed
    Error(AccountControllerErrorStateReason),
}

impl fmt::Display for AccountControllerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Offline => {
                write!(f, "Offline")
            }
            Self::Syncing => {
                write!(f, "Syncing")
            }
            Self::LoggedOut => {
                write!(f, "Logged Out")
            }
            Self::ReadyToConnect => {
                write!(f, "Ready to connect")
            }
            Self::Decentralised => {
                write!(f, "Decentralised")
            }
            Self::PendingSubscription => {
                write!(f, "Pending Subscription")
            }
            Self::Error(reason) => write!(f, "Error : {reason}"),
        }
    }
}
