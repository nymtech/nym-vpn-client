// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) mod register_device;
pub(crate) mod request_zknym;
pub(crate) mod sync_account;
pub(crate) mod sync_device;

use nym_vpn_lib_types::{
    AccountCommandError, RegisterDeviceError, RequestZkNymError, SyncAccountError, SyncDeviceError,
};
use nym_vpn_store::mnemonic::Mnemonic;
use request_zknym::RequestZkNymSummary;

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use nym_vpn_api_client::response::{NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnUsage};
use tokio::sync::oneshot;

use crate::{shared_state::DeviceState, AvailableTicketbooks, Error};

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

#[derive(Debug)]
pub struct ReturnSender<T, E> {
    sender: oneshot::Sender<Result<T, E>>,
}

impl<T, E> ReturnSender<T, E>
where
    T: std::fmt::Debug,
    E: std::fmt::Debug,
{
    pub fn new() -> (Self, oneshot::Receiver<Result<T, E>>) {
        let (sender, receiver) = oneshot::channel();
        (Self { sender }, receiver)
    }

    pub fn send(self, response: Result<T, E>)
    where
        T: Send,
        E: Send,
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
    StoreAccount(ReturnSender<(), AccountCommandError>, Mnemonic),
    ForgetAccount(ReturnSender<(), AccountCommandError>),
    SyncAccountState(Option<ReturnSender<NymVpnAccountSummaryResponse, SyncAccountError>>),
    SyncDeviceState(Option<ReturnSender<DeviceState, SyncDeviceError>>),
    GetUsage(ReturnSender<Vec<NymVpnUsage>, AccountCommandError>),
    GetDeviceIdentity(ReturnSender<String, AccountCommandError>),
    RegisterDevice(Option<ReturnSender<NymVpnDevice, RegisterDeviceError>>),
    GetDevices(ReturnSender<Vec<NymVpnDevice>, AccountCommandError>),
    GetActiveDevices(ReturnSender<Vec<NymVpnDevice>, AccountCommandError>),
    RequestZkNym(Option<ReturnSender<RequestZkNymSummary, RequestZkNymError>>),
    GetDeviceZkNym,
    GetZkNymsAvailableForDownload,
    GetZkNymById(String),
    ConfirmZkNymIdDownloaded(String),
    GetAvailableTickets(ReturnSender<AvailableTicketbooks, AccountCommandError>),
    SetStaticApiAddresses(
        ReturnSender<(), AccountCommandError>,
        Option<Vec<SocketAddr>>,
    ),
}

impl AccountCommand {
    pub fn kind(&self) -> String {
        self.to_string()
    }

    pub fn return_no_account(self, error: Error) {
        tracing::warn!("No account found: {error}");
        match self {
            AccountCommand::SyncAccountState(Some(tx)) => {
                tx.send(Err(SyncAccountError::NoAccountStored));
            }
            AccountCommand::SyncDeviceState(Some(tx)) => {
                tx.send(Err(SyncDeviceError::NoAccountStored));
            }
            AccountCommand::RegisterDevice(Some(tx)) => {
                tx.send(Err(RegisterDeviceError::NoAccountStored));
            }
            AccountCommand::RequestZkNym(Some(tx)) => {
                tx.send(Err(RequestZkNymError::NoAccountStored));
            }
            _ => {}
        }
    }

    pub fn return_no_device(self, error: Error) {
        tracing::warn!("No device found: {error}");
        match self {
            AccountCommand::SyncDeviceState(Some(tx)) => {
                tx.send(Err(SyncDeviceError::NoDeviceStored));
            }
            AccountCommand::RegisterDevice(Some(tx)) => {
                tx.send(Err(RegisterDeviceError::NoDeviceStored));
            }
            AccountCommand::RequestZkNym(Some(tx)) => {
                tx.send(Err(RequestZkNymError::NoDeviceStored));
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub(crate) enum AccountCommandResult {
    SyncAccountState(Result<NymVpnAccountSummaryResponse, SyncAccountError>),
    SyncDeviceState(Result<DeviceState, SyncDeviceError>),
    RegisterDevice(Result<NymVpnDevice, RegisterDeviceError>),
    RequestZkNym(Result<RequestZkNymSummary, RequestZkNymError>),
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
