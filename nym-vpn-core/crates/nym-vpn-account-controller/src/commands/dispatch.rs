// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityHandle;
use nym_vpn_lib_types::{
    AccountCommandError, ForgetAccountError, RegisterAccountError, RegisterDeviceError,
    RequestZkNymError, StoreAccountError, SyncAccountError, SyncDeviceError,
};
use nym_vpn_store::mnemonic::Mnemonic;

use std::net::SocketAddr;

use nym_vpn_api_client::{
    response::{NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnUsage},
    types::{Platform, VpnApiTimeSynced},
};
use tokio::sync::oneshot;

use crate::{
    AvailableTicketbooks, Error, PaymentResponse,
    commands::tasks::request_zknym::RequestZkNymSummary, shared_state::DeviceState,
};

#[derive(Debug, strum::Display)]
pub enum AccountCommand {
    StoreAccount(ReturnSender<(), StoreAccountError>, Mnemonic),
    RegisterAccount(
        ReturnSender<PaymentResponse, RegisterAccountError>,
        Platform,
    ),
    ForgetAccount(ReturnSender<(), ForgetAccountError>),
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
    RegisterOfflineMonitor(ReturnSender<(), AccountCommandError>, ConnectivityHandle),
    CheckDeviceTimeSync(ReturnSender<VpnApiTimeSynced, AccountCommandError>),
}

impl AccountCommand {
    pub fn kind(&self) -> String {
        self.to_string()
    }

    pub fn return_no_account(self, error: Error) {
        tracing::debug!("No account found: {error}");
        match self {
            AccountCommand::SyncAccountState(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(SyncAccountError::NoAccountStored));
                } else {
                    tracing::debug!("No account found during background account sync");
                }
            }
            AccountCommand::SyncDeviceState(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(SyncDeviceError::NoAccountStored));
                } else {
                    tracing::debug!("No account found during background device sync");
                }
            }
            AccountCommand::RegisterDevice(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(RegisterDeviceError::NoAccountStored));
                } else {
                    tracing::debug!("No account found during background device registration");
                }
            }
            AccountCommand::RequestZkNym(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(RequestZkNymError::NoAccountStored));
                } else {
                    tracing::debug!("No account found during background zk-nym request");
                }
            }
            _ => {
                tracing::error!("Command does not support no account: {self}");
            }
        }
    }

    pub fn return_no_device(self, error: Error) {
        tracing::debug!("No device found: {error}");
        match self {
            AccountCommand::SyncDeviceState(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(SyncDeviceError::NoDeviceStored));
                } else {
                    tracing::debug!("No device found during background device sync");
                }
            }
            AccountCommand::RegisterDevice(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(RegisterDeviceError::NoDeviceStored));
                } else {
                    tracing::debug!("No device found during background device registration");
                }
            }
            AccountCommand::RequestZkNym(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(RequestZkNymError::NoDeviceStored));
                } else {
                    tracing::debug!("No device found during background zk-nym request");
                }
            }
            _ => {
                tracing::error!("Command does not support no device: {self}");
            }
        }
    }

    pub fn return_no_connectivity(self) {
        tracing::debug!("No connectivity");
        match self {
            AccountCommand::SyncAccountState(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(SyncAccountError::Offline));
                } else {
                    tracing::debug!("No connectivity during background account sync");
                }
            }
            AccountCommand::SyncDeviceState(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(SyncDeviceError::Offline));
                } else {
                    tracing::debug!("No connectivity during background device sync");
                }
            }
            AccountCommand::RegisterDevice(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(RegisterDeviceError::Offline));
                } else {
                    tracing::debug!("No connectivity during background device registration");
                }
            }
            AccountCommand::RequestZkNym(tx) => {
                if let Some(tx) = tx {
                    tx.send(Err(RequestZkNymError::Offline));
                } else {
                    tracing::debug!("No connectivity during background zk-nym request");
                }
            }
            _ => {
                tracing::error!("Command does not support offline mode: {self}");
            }
        }
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
