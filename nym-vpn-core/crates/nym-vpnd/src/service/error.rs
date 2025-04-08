// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::tunnel_state_machine::Error as TunnelStateMachineError;
use tracing::error;

use super::config::ConfigSetupError;

#[derive(Debug, thiserror::Error)]
pub enum AccountControllerError {
    #[error("failed to init account controller: {reason}")]
    Initialization { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum SetNetworkError {
    #[error("failed to read config")]
    ReadConfig {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to write config")]
    WriteConfig {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to set network: {0}")]
    NetworkNotFound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AccountLinksError {
    #[error("account management not configured")]
    AccountManagementNotConfigured,

    #[error("failed to parse account management paths")]
    FailedToParseAccountLinks,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceDeleteLogFileError {
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("account error: {0}")]
    AccountController(#[from] AccountControllerError),

    #[error("config setup error: {0}")]
    ConfigSetup(#[source] ConfigSetupError),

    #[error("state machine error: {0}")]
    StateMachine(#[source] TunnelStateMachineError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
