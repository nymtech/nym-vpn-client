// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) mod register_device;
pub(crate) mod request_zknym;
pub(crate) mod sync_account;
pub(crate) mod sync_device;

use nym_vpn_store::mnemonic::Mnemonic;
pub use register_device::RegisterDeviceError;
use request_zknym::RequestZkNymSummary;
pub use request_zknym::{RequestZkNymError, RequestZkNymSuccess};

use std::{collections::HashMap, sync::Arc};

use nym_vpn_api_client::response::{NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnUsage};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::{shared_state::DeviceState, AvailableTicketbooks};

#[derive(Debug, Default)]
pub(crate) struct RunningCommands {
    running_commands: Arc<tokio::sync::Mutex<HashMap<String, Vec<AccountCommand>>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    IsFirst,
    IsNotFirst,
}

// Add the command to the set of running commands.
// Returns true if this is the first command of this type, otherwise false.
impl RunningCommands {
    pub(crate) async fn add(&self, command: AccountCommand) -> Command {
        let mut running_commands = self.running_commands.lock().await;
        let commands = running_commands.entry(command.kind()).or_default();
        let is_first = if commands.is_empty() {
            Command::IsFirst
        } else {
            Command::IsNotFirst
        };
        commands.push(command);
        is_first
    }

    pub(crate) async fn remove(&self, command: &AccountCommand) -> Vec<AccountCommand> {
        let mut running_commands = self.running_commands.lock().await;
        let removed_commands = running_commands.remove(&command.kind());
        removed_commands.unwrap_or_default()
    }
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AccountCommandError {
    #[error("failed to sync account state: {0}")]
    SyncAccountEndpointFailure(VpnApiEndpointFailure),

    #[error("failed to sync device state: {0}")]
    SyncDeviceEndpointFailure(VpnApiEndpointFailure),

    #[error("failed to register device: {0}")]
    RegisterDeviceEndpointFailure(VpnApiEndpointFailure),

    #[error("failed to request zk nym")]
    RequestZkNym {
        successes: Vec<RequestZkNymSuccess>,
        failed: Vec<RequestZkNymError>,
    },

    #[error("failed to request zk nym")]
    RequestZkNymGeneral(RequestZkNymError),

    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("device registration is in progress")]
    RegistrationInProgress,

    #[error("failed to remove account: {0}")]
    RemoveAccount(String),

    #[error("failed to remove device from nym vpn api: {0}")]
    UnregisterDeviceApiClientFailure(String),

    #[error("failed to remove device identity: {0}")]
    RemoveDeviceIdentity(String),

    #[error("failed to reset credential storage: {0}")]
    ResetCredentialStorage(String),

    #[error("failed to remove account files: {0}")]
    RemoveAccountFiles(String),

    #[error("failed to init device keys: {0}")]
    InitDeviceKeys(String),

    // Catch all for any other error
    #[error("general error: {0}")]
    General(String),

    // Internal error that should not happen
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<RegisterDeviceError> for AccountCommandError {
    fn from(err: RegisterDeviceError) -> Self {
        match err {
            RegisterDeviceError::RegisterDeviceEndpointFailure(failure) => {
                AccountCommandError::RegisterDeviceEndpointFailure(failure)
            }
            RegisterDeviceError::General(message) => AccountCommandError::General(message),
        }
    }
}

impl From<RequestZkNymError> for AccountCommandError {
    fn from(err: RequestZkNymError) -> Self {
        AccountCommandError::RequestZkNymGeneral(err)
    }
}

impl AccountCommandError {
    pub fn internal(message: impl ToString) -> Self {
        AccountCommandError::Internal(message.to_string())
    }

    pub fn general(message: impl ToString) -> Self {
        AccountCommandError::General(message.to_string())
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[error("nym-vpn-api error: message={message}, message_id={message_id:?}, code_reference_id={code_reference_id:?}")]
pub struct VpnApiEndpointFailure {
    pub message: String,
    pub message_id: Option<String>,
    pub code_reference_id: Option<String>,
}

impl From<nym_vpn_api_client::response::NymErrorResponse> for VpnApiEndpointFailure {
    fn from(response: nym_vpn_api_client::response::NymErrorResponse) -> Self {
        Self {
            message: response.message,
            message_id: response.message_id,
            code_reference_id: response.code_reference_id,
        }
    }
}

impl TryFrom<nym_vpn_api_client::VpnApiClientError> for VpnApiEndpointFailure {
    type Error = nym_vpn_api_client::VpnApiClientError;

    fn try_from(response: nym_vpn_api_client::VpnApiClientError) -> Result<Self, Self::Error> {
        nym_vpn_api_client::response::extract_error_response(&response)
            .map(VpnApiEndpointFailure::from)
            .ok_or(response)
    }
}

#[derive(Debug)]
pub struct ReturnSender<T> {
    sender: oneshot::Sender<Result<T, AccountCommandError>>,
}

impl<T> ReturnSender<T>
where
    T: std::fmt::Debug,
{
    pub fn new() -> (Self, oneshot::Receiver<Result<T, AccountCommandError>>) {
        let (sender, receiver) = oneshot::channel();
        (Self { sender }, receiver)
    }

    pub fn send(self, response: Result<T, AccountCommandError>)
    where
        T: Send,
    {
        self.sender
            .send(response)
            .inspect_err(|err| {
                tracing::error!("Failed to send response: {:#?}", err);
            })
            .ok();
    }
}

#[derive(Debug, strum::Display)]
pub enum AccountCommand {
    StoreAccount(ReturnSender<()>, Mnemonic),
    ForgetAccount(ReturnSender<()>),
    SyncAccountState(Option<ReturnSender<NymVpnAccountSummaryResponse>>),
    SyncDeviceState(Option<ReturnSender<DeviceState>>),
    GetUsage(ReturnSender<Vec<NymVpnUsage>>),
    GetDeviceIdentity(ReturnSender<String>),
    RegisterDevice(Option<ReturnSender<NymVpnDevice>>),
    GetDevices(ReturnSender<Vec<NymVpnDevice>>),
    GetActiveDevices(ReturnSender<Vec<NymVpnDevice>>),
    RequestZkNym(Option<ReturnSender<RequestZkNymSummary>>),
    GetDeviceZkNym,
    GetZkNymsAvailableForDownload,
    GetZkNymById(String),
    ConfirmZkNymIdDownloaded(String),
    GetAvailableTickets(ReturnSender<AvailableTicketbooks>),
}

impl AccountCommand {
    pub fn kind(&self) -> String {
        self.to_string()
    }

    pub fn return_error(self, error: AccountCommandError) {
        tracing::warn!("Returning error: {:?}", error);
        match self {
            AccountCommand::SyncAccountState(Some(tx)) => {
                tx.send(Err(error));
            }
            AccountCommand::SyncDeviceState(Some(tx)) => {
                tx.send(Err(error));
            }
            AccountCommand::RegisterDevice(Some(tx)) => {
                tx.send(Err(error));
            }
            AccountCommand::RequestZkNym(Some(tx)) => {
                tx.send(Err(error));
            }
            _ => {}
        }
    }
}

// WIP: Fix this clippy
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum AccountCommandResult {
    SyncAccountState(Result<NymVpnAccountSummaryResponse, AccountCommandError>),
    SyncDeviceState(Result<DeviceState, AccountCommandError>),
    RegisterDevice(Result<NymVpnDevice, AccountCommandError>),
    RequestZkNym(Result<RequestZkNymSummary, AccountCommandError>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_command_kind_representation() {
        assert_eq!(
            AccountCommand::SyncAccountState(None).kind(),
            "SyncAccountState"
        );
        assert_eq!(
            AccountCommand::SyncDeviceState(None).kind(),
            "SyncDeviceState"
        );
        assert_eq!(
            AccountCommand::RegisterDevice(None).kind(),
            "RegisterDevice"
        );
        assert_eq!(AccountCommand::RequestZkNym(None).kind(), "RequestZkNym");
        assert_eq!(AccountCommand::GetDeviceZkNym.kind(), "GetDeviceZkNym");
        assert_eq!(
            AccountCommand::GetZkNymsAvailableForDownload.kind(),
            "GetZkNymsAvailableForDownload"
        );
        assert_eq!(
            AccountCommand::GetZkNymById("some_id".to_string()).kind(),
            "GetZkNymById"
        );
        assert_eq!(
            AccountCommand::ConfirmZkNymIdDownloaded("some_id".to_string()).kind(),
            "ConfirmZkNymIdDownloaded"
        );
        let (tx, _) = ReturnSender::new();
        assert_eq!(
            AccountCommand::GetAvailableTickets(tx).kind(),
            "GetAvailableTickets"
        );
    }
}
