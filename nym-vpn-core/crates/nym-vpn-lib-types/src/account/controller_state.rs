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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum AccountControllerState {
    /// We don't have network
    #[cfg_attr(feature = "cbor", n(0))]
    Offline,

    /// Figuring out account state
    #[cfg_attr(feature = "cbor", n(1))]
    Syncing,

    /// Not logged in with a mnemonic
    #[cfg_attr(feature = "cbor", n(2))]
    LoggedOut,

    /// Logged in, registered device
    #[cfg_attr(feature = "cbor", n(3))]
    ReadyToConnect,

    /// Logged in, operating independently of VPN API
    #[cfg_attr(feature = "cbor", n(4))]
    Decentralised,

    /// Logged in, subscription is pending (e.g. cash payment still processing)
    #[cfg_attr(feature = "cbor", n(5))]
    PendingSubscription,

    /// Logged in, error during sync, can't proceed
    #[cfg_attr(feature = "cbor", n(6))]
    Error(#[cfg_attr(feature = "cbor", n(0))] AccountControllerErrorStateReason),
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
