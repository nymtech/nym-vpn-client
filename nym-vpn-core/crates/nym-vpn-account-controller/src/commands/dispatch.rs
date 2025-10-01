// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{AccountCommandError, RegisterAccountResponse};
use nym_vpn_store::account::StorableAccount;

use std::net::SocketAddr;

use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::Platform,
};
use tokio::sync::oneshot;

use crate::AvailableTicketbooks;

#[derive(Debug, strum::Display)]
pub enum AccountCommand {
    /// Generate a mnemonic and store it
    CreateAccount(ReturnSender<(), AccountCommandError>),

    /// Store the given account
    StoreAccount(ReturnSender<(), AccountCommandError>, StorableAccount),

    /// Register the given account (meant to take the stored mnemonic). DOES NOT STORE IT. This is only used my mobile for IAP at the moment
    RegisterAccount(
        ReturnSender<RegisterAccountResponse, AccountCommandError>,
        StorableAccount,
        Platform,
    ),

    /// Delete the stored account and every associated data
    ForgetAccount(ReturnSender<(), AccountCommandError>),

    /// Reset the device identity, optionally take a seed for reproducibility
    ResetDeviceIdentity(ReturnSender<(), AccountCommandError>, Option<[u8; 32]>),

    /// Forces the AC to sync with the VPN API
    RefreshAccountState(ReturnSender<(), AccountCommandError>),

    /// Tells the AC it's firewalled off the VPN API, so it should stop/pause network communication
    VpnApiFirewallUp(ReturnSender<(), AccountCommandError>),

    /// Tells the AC free to go ahead
    VpnApiFirewallDown(ReturnSender<(), AccountCommandError>),

    /// Read-only commands
    Common(CommonCommand),
}

/// These commands have no impact on the state. Handling can be grouped in some cases
#[derive(Debug, strum::Display)]
pub enum CommonCommand {
    /// Returns Some(account) if an account is stored, None otherwise
    GetStoredAccount(ReturnSender<Option<StorableAccount>, AccountCommandError>),

    /// Returns Some(address) if an account is stored, None otherwise
    GetAccountIdentity(ReturnSender<Option<String>, AccountCommandError>),

    /// Returns Some(id) if the current device has an identity (is registered), None otherwise
    GetDeviceIdentity(ReturnSender<Option<String>, AccountCommandError>),

    /// Returns the state of the account
    GetUsage(ReturnSender<Vec<NymVpnUsage>, AccountCommandError>),

    /// Get the list of devices registered to that account
    GetDevices(ReturnSender<Vec<NymVpnDevice>, AccountCommandError>),

    /// Get the list of active devices registered to that account
    GetActiveDevices(ReturnSender<Vec<NymVpnDevice>, AccountCommandError>),

    /// Returns a list of tickets available in storage
    GetAvailableTickets(ReturnSender<AvailableTicketbooks, AccountCommandError>),

    /// Override the VPN API client resolver to allow him to go through the firewall
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
