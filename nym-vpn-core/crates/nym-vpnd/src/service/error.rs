// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::tunnel_state_machine::Error as TunnelStateMachineError;
use tracing::error;

use super::config::ConfigSetupError;

#[derive(Debug, thiserror::Error)]
pub enum AccountControllerError {
    #[error("Failed to init account controller: {reason}")]
    Initialization { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum SetNetworkError {
    #[error("Failed to read config")]
    ReadConfig {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to write config")]
    WriteConfig {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to set network: {0}")]
    NetworkNotFound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AccountLinksError {
    #[error("Account management not configured")]
    AccountManagementNotConfigured,

    #[error("Failed to parse account management paths")]
    FailedToParseAccountLinks,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceDeleteLogFileError {
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Account controller error")]
    AccountController(#[from] AccountControllerError),

    #[error("Config setup error")]
    ConfigSetup(#[source] ConfigSetupError),

    #[error("State machine error")]
    StateMachine(#[source] TunnelStateMachineError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
