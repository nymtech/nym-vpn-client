// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::tunnel_state_machine::Error as TunnelStateMachineError;
use nym_vpn_lib_types::AccountCommandError;
use tracing::error;

use super::config::ConfigSetupError;

// Failure to initiate the connect
#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceConnectError {
    #[error("internal error: {0}")]
    Internal(String),

    #[error("connection attempt cancelled")]
    Cancel,
}

// Failure to initiate the disconnect
#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceDisconnectError {
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    // Failures related to the operational aspects of the account controller
    #[error(transparent)]
    AccountController {
        #[from]
        source: nym_vpn_account_controller::Error,
    },

    // Failures for commands run by the account controller
    #[error(transparent)]
    AccountCommand {
        #[from]
        source: AccountCommandError,
    },

    #[error("invalid mnemonic")]
    InvalidMnemonic {
        #[from]
        source: bip39::Error,
    },

    #[error("failed to reset device keys")]
    FailedToResetDeviceKeys {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("account not configured")]
    AccountManagementNotConfigured,

    #[error("failed to parse account links")]
    FailedToParseAccountLinks,

    #[error("unable to proceed while connected")]
    IsConnected,
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

#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceDeleteLogFileError {
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("account error: {0}")]
    Account(#[from] AccountError),

    #[error("config setup error: {0}")]
    ConfigSetup(#[source] ConfigSetupError),

    #[error("state machine error: {0}")]
    StateMachine(#[source] TunnelStateMachineError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
