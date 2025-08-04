// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    AccountCommandError, CreateAccountError, ForgetAccountError, GetMnemonicError,
    RegisterAccountError, StoreAccountError, SyncAccountError,
};
use nym_vpn_store::mnemonic::Mnemonic;

use std::net::SocketAddr;

use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::Platform,
};
use tokio::sync::oneshot;

use crate::{AvailableTicketbooks, RegisterAccountResponse};

#[derive(Debug, strum::Display)]
pub enum AccountCommand {
    CreateAccount(ReturnSender<(), CreateAccountError>), // Generate a mnemonic and store it
    StoreAccount(ReturnSender<(), StoreAccountError>, Mnemonic), // Store the given mnemonic (optional API check)
    RegisterAccount(
        // Register the given mnemnonic (meant to take the sotred mnemonic). DOES NOT STORE IT
        ReturnSender<RegisterAccountResponse, RegisterAccountError>,
        Mnemonic,
        Platform,
    ),
    ForgetAccount(ReturnSender<(), ForgetAccountError>),
    ResetDeviceIdentity(ReturnSender<(), AccountCommandError>, Option<[u8; 32]>), // SW maybe new error type?

    RefreshAccountState(ReturnSender<(), SyncAccountError>), // SW Rename error

    Common(CommonCommand),
}

/// These commands have no impact on the state. Handling can be grouped in some cases
#[derive(Debug, strum::Display)]
pub enum CommonCommand {
    GetStoredMnemonic(ReturnSender<Mnemonic, GetMnemonicError>),
    GetAccountIdentity(ReturnSender<Option<String>, AccountCommandError>),
    GetDeviceIdentity(ReturnSender<String, AccountCommandError>),
    GetUsage(ReturnSender<Vec<NymVpnUsage>, AccountCommandError>),
    GetDevices(ReturnSender<Vec<NymVpnDevice>, AccountCommandError>),
    GetActiveDevices(ReturnSender<Vec<NymVpnDevice>, AccountCommandError>),
    GetAvailableTickets(ReturnSender<AvailableTicketbooks, AccountCommandError>),
    SetStaticApiAddresses(
        ReturnSender<(), AccountCommandError>,
        Option<Vec<SocketAddr>>,
    ),
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
