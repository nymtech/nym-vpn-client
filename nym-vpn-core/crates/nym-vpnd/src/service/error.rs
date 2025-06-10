// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::{
    storage::VpnClientOnDiskStorageError, tunnel_state_machine::Error as TunnelStateMachineError,
};
use tracing::error;

use super::config::ConfigSetupError;

#[derive(Debug, thiserror::Error)]
pub enum AccountControllerError {
    #[error("failed to init account controller: {reason}")]
    Initialization { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum StatisticsControllerError {
    #[error("failed to init statistics controller: {reason}")]
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
    #[error("account controller error")]
    AccountController(#[from] AccountControllerError),

    #[error("statistics error: {0}")]
    StatisticsController(#[from] StatisticsControllerError),

    #[error("storage error: {0}")]
    Storage(#[from] VpnClientOnDiskStorageError),

    #[error("config setup error")]
    ConfigSetup(#[source] ConfigSetupError),

    #[error("state machine error")]
    StateMachine(#[source] TunnelStateMachineError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
