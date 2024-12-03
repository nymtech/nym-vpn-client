// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) mod register_device;
pub(crate) mod update_account;
pub(crate) mod update_device;
pub(crate) mod zknym;

use std::{collections::HashMap, sync::Arc};

use nym_vpn_api_client::response::{NymVpnAccountSummaryResponse, NymVpnDevice};
use tokio::sync::oneshot;

use crate::{error::Error, shared_state::DeviceState, AvailableTicketbooks};

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
    #[error("failed to update account state: {message}")]
    UpdateAccountEndpointFailure {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
        base_url: Box<url::Url>,
    },

    #[error("failed to update device state: {message}")]
    UpdateDeviceEndpointFailure {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("failed to register device: {message}")]
    RegisterDeviceEndpointFailure {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    // Catch all for any other error
    #[error("general error: {0}")]
    General(String),

    // Internal error that should not happen
    #[error("internal error: {0}")]
    Internal(String),
}

impl AccountCommandError {
    pub fn internal(message: impl ToString) -> Self {
        AccountCommandError::Internal(message.to_string())
    }

    pub fn general(message: impl ToString) -> Self {
        AccountCommandError::General(message.to_string())
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
    ResetAccount,
    UpdateAccountState(Option<ReturnSender<NymVpnAccountSummaryResponse>>),
    UpdateDeviceState(Option<ReturnSender<DeviceState>>),
    RegisterDevice(Option<ReturnSender<NymVpnDevice>>),
    RequestZkNym,
    GetDeviceZkNym,
    GetZkNymsAvailableForDownload,
    GetZkNymById(String),
    ConfirmZkNymIdDownloaded(String),
    GetAvailableTickets(oneshot::Sender<Result<AvailableTicketbooks, Error>>),
}

impl AccountCommand {
    pub fn kind(&self) -> String {
        self.to_string()
    }

    pub fn return_error(self, error: AccountCommandError) {
        tracing::warn!("Returning error: {:?}", error);
        match self {
            AccountCommand::UpdateAccountState(Some(tx)) => {
                tx.send(Err(error));
            }
            AccountCommand::UpdateDeviceState(Some(tx)) => {
                tx.send(Err(error));
            }
            AccountCommand::RegisterDevice(Some(tx)) => {
                tx.send(Err(error));
            }
            //AccountCommand::GetAvailableTickets(tx) => {
            //    tx.send(Err(error))
            //        .inspect_err(|err| {
            //            tracing::error!("Failed to send error response: {:?}", err);
            //        })
            //        .ok();
            //}
            _ => {}
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum AccountCommandResult {
    UpdateAccountState(Result<NymVpnAccountSummaryResponse, AccountCommandError>),
    UpdateDeviceState(Result<DeviceState, AccountCommandError>),
    RegisterDevice(Result<NymVpnDevice, AccountCommandError>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_command_kind_representation() {
        assert_eq!(
            AccountCommand::UpdateAccountState(None).kind(),
            "UpdateAccountState"
        );
        assert_eq!(
            AccountCommand::UpdateDeviceState(None).kind(),
            "UpdateDeviceState"
        );
        assert_eq!(
            AccountCommand::RegisterDevice(None).kind(),
            "RegisterDevice"
        );
        assert_eq!(AccountCommand::RequestZkNym.kind(), "RequestZkNym");
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
        assert_eq!(
            AccountCommand::GetAvailableTickets(oneshot::channel().0).kind(),
            "GetAvailableTickets"
        );
    }
}
