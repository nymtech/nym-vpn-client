// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::AccountCommandError;
use nym_vpn_lib::tunnel_state_machine::Error as TunnelStateMachineError;
use tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError};
use tracing::error;

use super::config::ConfigSetupError;

// Failure to initiate the connect
#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceConnectError {
    #[error("internal error: {0}")]
    Internal(String),

    #[error("failed to connect: {0}")]
    Account(#[from] AccountNotReady),

    #[error("connection attempt cancelled")]
    Cancel,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum AccountNotReady {
    #[error("update account failed: {message}")]
    UpdateAccount {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("update device failed: {message}")]
    UpdateDevice {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("register device failed: {message}")]
    RegisterDevice {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("no account stored")]
    NoAccountStored,

    #[error("no device identity stored")]
    NoDeviceStored,

    // There are usually multiple independent zknym requests at a time
    #[error("failed to request zk-nym(s)")]
    RequestZkNym {
        failed: Vec<nym_vpn_account_controller::RequestZkNymError>,
    },

    #[error("general error: {0}")]
    General(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<AccountCommandError> for AccountNotReady {
    fn from(err: AccountCommandError) -> Self {
        match err {
            AccountCommandError::SyncAccountEndpointFailure(e) => AccountNotReady::UpdateAccount {
                message: e.message,
                message_id: e.message_id,
                code_reference_id: e.code_reference_id,
            },
            AccountCommandError::SyncDeviceEndpointFailure(e) => AccountNotReady::UpdateDevice {
                message: e.message,
                message_id: e.message_id,
                code_reference_id: e.code_reference_id,
            },
            AccountCommandError::RegisterDeviceEndpointFailure(e) => {
                AccountNotReady::RegisterDevice {
                    message: e.message,
                    message_id: e.message_id,
                    code_reference_id: e.code_reference_id,
                }
            }
            AccountCommandError::RequestZkNym {
                successes: _,
                failed,
            } => AccountNotReady::RequestZkNym { failed },
            AccountCommandError::RequestZkNymGeneral(e) => {
                AccountNotReady::RequestZkNym { failed: vec![e] }
            }
            AccountCommandError::NoAccountStored => AccountNotReady::NoAccountStored,
            AccountCommandError::NoDeviceStored => AccountNotReady::NoDeviceStored,
            AccountCommandError::RemoveAccount(e) => AccountNotReady::General(e),
            AccountCommandError::RemoveDeviceIdentity(e) => AccountNotReady::General(e),
            AccountCommandError::ResetCredentialStorage(e) => AccountNotReady::General(e),
            AccountCommandError::RemoveAccountFiles(e) => AccountNotReady::General(e),
            AccountCommandError::InitDeviceKeys(e) => AccountNotReady::General(e),
            AccountCommandError::General(err) => AccountNotReady::General(err),
            AccountCommandError::Internal(err) => AccountNotReady::Internal(err),
            AccountCommandError::UnregisterDeviceApiClientFailure(err) => {
                AccountNotReady::Internal(err)
            }
            AccountCommandError::RegistrationInProgress => {
                AccountNotReady::Internal(err.to_string())
            }
        }
    }
}

// Failure to initiate the disconnect
#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceDisconnectError {
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("invalid mnemonic")]
    InvalidMnemonic {
        #[from]
        source: bip39::Error,
    },

    #[error("failed to store account: {source}")]
    FailedToStoreAccount {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to check if account is stored: {source}")]
    FailedToCheckIfAccountIsStored {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to remove account: {source}")]
    FailedToRemoveAccount {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to forget account: {source}")]
    FailedToForgetAccount {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to load account: {source}")]
    FailedToLoadAccount {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("no nym-vpn-api url setup")]
    MissingApiUrl,

    #[error("invalid nym-vpn-api url")]
    InvalidApiUrl,

    #[error(transparent)]
    VpnApiClientError(#[from] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to load keys: {source}")]
    FailedToLoadKeys {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to get account summary")]
    FailedToGetAccountSummary,

    #[error("failed to send command")]
    SendCommand {
        source: Box<SendError<nym_vpn_account_controller::AccountCommand>>,
    },

    #[error("account controller not ready to handle command")]
    RecvCommand { source: Box<RecvError> },

    #[error("no account stored")]
    NoAccountStored,

    #[error("failed to init device keys")]
    FailedToInitDeviceKeys {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to reset device keys")]
    FailedToResetDeviceKeys {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error(transparent)]
    AccountControllerError {
        source: nym_vpn_account_controller::Error,
    },

    #[error(transparent)]
    AccountCommandError {
        source: nym_vpn_account_controller::AccountCommandError,
    },

    #[error("account not configured")]
    AccountManagementNotConfigured,

    #[error("failed to parse account links")]
    FailedToParseAccountLinks,

    #[error("timeout: {0}")]
    Timeout(String),

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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("account error: {0}")]
    Account(#[source] AccountError),

    #[error("config setup error: {0}")]
    ConfigSetup(#[source] ConfigSetupError),

    #[error("state machine error: {0}")]
    StateMachine(#[source] TunnelStateMachineError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
