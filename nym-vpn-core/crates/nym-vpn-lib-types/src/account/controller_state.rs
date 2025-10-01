// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use crate::AccountControllerErrorStateReason;

// Public enum describing the tunnel state
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AccountControllerState {
    /// We don't have network
    Offline,

    /// Figuring out account state
    Syncing,

    /// Not logged in with a mnemonic
    LoggedOut,

    /// Logged in, registered device, available zk-nyms
    ReadyToConnect,

    /// Logged in, operating independently of VPN API
    Decentralised,

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
            Self::Error(reason) => write!(f, "Error : {reason}"),
        }
    }
}
