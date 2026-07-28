// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::deeplink::{CreateDeeplinkParams, DeeplinkMnemonic};
use nym_validator_client::nyxd::Coin;
use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::Platform,
};
use nym_vpn_lib_types::{
    AccountCommandError, AutologinResponse, RegisterAccountResponse, StorableAccount,
    StoredAccountMode, VpnAccountSummary,
};
use tokio::sync::oneshot;

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

    /// Link another account with the currently logged-on API account
    LinkAccount(ReturnSender<(), AccountCommandError>, StorableAccount),

    /// Rotate the wireguard keys
    RotateKeys(ReturnSender<(), AccountCommandError>),

    /// Retrieve current, on-chain, balance of the account. Only applicable for decentralised accounts
    AccountBalance(ReturnSender<Vec<Coin>, AccountCommandError>),

    /// Attempt to obtain one ticketbook (per type) for the decentralised account
    ObtainTicketbooks(ReturnSender<(), AccountCommandError>),

    /// Reset the device identity, optionally take a seed for reproducibility
    ResetDeviceIdentity(ReturnSender<(), AccountCommandError>, Option<[u8; 32]>),

    /// Re-evaluates the account state
    RefreshAccountState(ReturnSender<(), AccountCommandError>, bool),

    /// Tells the AC it's firewalled off the VPN API, so it should stop/pause network communication
    VpnApiFirewallUp(ReturnSender<(), AccountCommandError>),

    /// Tells the AC free to go ahead
    VpnApiFirewallDown(ReturnSender<(), AccountCommandError>),

    /// Read-only commands
    Common(CommonCommand),
}

impl AccountCommand {
    pub fn return_error(self, error: AccountCommandError) {
        match self {
            AccountCommand::CreateAccount(return_sender) => return_sender.send(Err(error)),
            AccountCommand::StoreAccount(return_sender, _) => return_sender.send(Err(error)),
            AccountCommand::RegisterAccount(return_sender, _, _) => return_sender.send(Err(error)),
            AccountCommand::ForgetAccount(return_sender) => return_sender.send(Err(error)),
            AccountCommand::LinkAccount(return_sender, _) => return_sender.send(Err(error)),
            AccountCommand::RotateKeys(return_sender) => return_sender.send(Err(error)),
            AccountCommand::AccountBalance(return_sender) => return_sender.send(Err(error)),
            AccountCommand::ObtainTicketbooks(return_sender) => return_sender.send(Err(error)),
            AccountCommand::ResetDeviceIdentity(return_sender, _) => return_sender.send(Err(error)),
            AccountCommand::RefreshAccountState(return_sender, _) => return_sender.send(Err(error)),
            AccountCommand::VpnApiFirewallUp(return_sender) => return_sender.send(Err(error)),
            AccountCommand::VpnApiFirewallDown(return_sender) => return_sender.send(Err(error)),
            AccountCommand::Common(common_command) => match common_command {
                CommonCommand::GetStoredAccount(return_sender) => return_sender.send(Err(error)),
                CommonCommand::GetAccountIdentity(return_sender) => return_sender.send(Err(error)),
                CommonCommand::GetCanonicalAccountIdentity(return_sender) => {
                    return_sender.send(Err(error))
                }
                CommonCommand::GetAccountMode(return_sender) => return_sender.send(Err(error)),
                CommonCommand::GetDeviceIdentity(return_sender) => return_sender.send(Err(error)),
                CommonCommand::GetUsage(return_sender) => return_sender.send(Err(error)),
                CommonCommand::GetDevices(return_sender) => return_sender.send(Err(error)),
                CommonCommand::GetActiveDevices(return_sender) => return_sender.send(Err(error)),
                CommonCommand::GetAccountSummary(return_sender) => return_sender.send(Err(error)),
                CommonCommand::GetDeeplink(return_sender, _) => return_sender.send(Err(error)),
                CommonCommand::GetAutologinDeeplink(return_sender, _) => {
                    return_sender.send(Err(error))
                }
                CommonCommand::DeriveDeeplinkMnemonic(return_sender, _) => {
                    return_sender.send(Err(error))
                }
            },
        }
    }
}

/// These commands have no impact on the state. Handling can be grouped in some cases
#[derive(Debug, strum::Display)]
pub enum CommonCommand {
    /// Returns Some(account) if an account is stored, None otherwise
    GetStoredAccount(ReturnSender<Option<StorableAccount>, AccountCommandError>),

    /// Returns Some(address) if an account is stored, None otherwise
    GetAccountIdentity(ReturnSender<Option<String>, AccountCommandError>),

    /// Returns the identifier of the canonical (API) account, or error.
    GetCanonicalAccountIdentity(ReturnSender<Option<String>, AccountCommandError>),

    /// Returns Some(mode) if an account is logged-in, None otherwise
    GetAccountMode(ReturnSender<Option<StoredAccountMode>, AccountCommandError>),

    /// Returns Some(id) if the current device has an identity (is registered), None otherwise
    GetDeviceIdentity(ReturnSender<Option<String>, AccountCommandError>),

    /// Returns the state of the account
    GetUsage(ReturnSender<Vec<NymVpnUsage>, AccountCommandError>),

    /// Get the list of devices registered to that account
    GetDevices(ReturnSender<Vec<NymVpnDevice>, AccountCommandError>),

    /// Get the list of active devices registered to that account
    GetActiveDevices(ReturnSender<Vec<NymVpnDevice>, AccountCommandError>),

    /// Returns the VPN account summary if the account is logged-in
    GetAccountSummary(ReturnSender<Option<VpnAccountSummary>, AccountCommandError>),

    /// Return the deeplink URL for the specfied deeplink kind and name
    GetDeeplink(
        ReturnSender<String, AccountCommandError>,
        CreateDeeplinkParams,
    ),

    /// Return the autologin deeplink URL for the specfied deeplink kind and name
    GetAutologinDeeplink(
        ReturnSender<AutologinResponse, AccountCommandError>,
        CreateDeeplinkParams,
    ),

    /// Derive the mnemonic from the deeplink callback URL
    DeriveDeeplinkMnemonic(ReturnSender<DeeplinkMnemonic, AccountCommandError>, String),
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
