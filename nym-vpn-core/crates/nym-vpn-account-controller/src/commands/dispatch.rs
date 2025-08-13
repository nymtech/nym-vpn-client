// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{AccountCommandError, RegisterAccountResponse};
use nym_vpn_store::mnemonic::Mnemonic;

use std::net::SocketAddr;

use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::Platform,
};
use tokio::sync::oneshot;

use crate::AvailableTicketbooks;

#[derive(Debug, strum::Display)]
pub enum AccountCommand {
    CreateAccount(ReturnSender<(), AccountCommandError>), // Generate a mnemonic and store it
    StoreAccount(ReturnSender<(), AccountCommandError>, Mnemonic), // Store the given mnemonic (optional API check)
    RegisterAccount(
        // Register the given mnemnonic (meant to take the sotred mnemonic). DOES NOT STORE IT
        ReturnSender<RegisterAccountResponse, AccountCommandError>,
        Mnemonic,
        Platform,
    ),
    ForgetAccount(ReturnSender<(), AccountCommandError>),
    ResetDeviceIdentity(ReturnSender<(), AccountCommandError>, Option<[u8; 32]>),

    RefreshAccountState(ReturnSender<(), AccountCommandError>),

    Common(CommonCommand),
}

/// These commands have no impact on the state. Handling can be grouped in some cases
#[derive(Debug, strum::Display)]
pub enum CommonCommand {
    GetStoredMnemonic(ReturnSender<Option<Mnemonic>, AccountCommandError>),
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
