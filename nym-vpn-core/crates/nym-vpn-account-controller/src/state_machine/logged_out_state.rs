// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_lib_types::AccountCommandError;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    SharedAccountState,
    commands::{AccountCommand, CommonCommand, ReturnSender, common_handler, handler},
    state_machine::{
        AccountControllerStateHandler, NextAccountControllerState, OfflineState,
        PrivateAccountControllerState, SyncingState,
    },
};

/// LoggedOut state
/// We are logged out
/// - We are online
/// - No account is stored
/// - This device does not have an identity
///
/// Our actions are limited, because most commands require an account.
///
///
/// Possible next state :
/// - SyncingState : A successful store_account command handling leads us into SyncingState, to determine where we are at
/// - OfflineState : the connectivity monitor is telling we're not connected
///
pub struct LoggedOutState;

impl LoggedOutState {
    pub fn enter<C: ConnectivityMonitor>() -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        (Box::new(Self), PrivateAccountControllerState::LoggedOut)
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for LoggedOutState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C> {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                // Intentionally no command grouping for clarity
                match command {
                    AccountCommand::CreateAccount(return_sender) => {
                        return_sender.send(handler::handle_create_account(shared_state).await);
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0));
                    }
                    AccountCommand::StoreAccount(return_sender, mnemonic) => {
                        if let Err(e) = handler::handle_store_account(shared_state, mnemonic).await{
                            return_sender.send(Err(e));
                            return NextAccountControllerState::SameState(self);
                        } else {
                            return_sender.send(Ok(()));
                            return NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0));
                        }
                    },
                    AccountCommand::ForgetAccount(return_sender) => return_sender.send(Ok(())),
                    AccountCommand::AccountBalance(return_sender) => return_no_account(return_sender),
                    AccountCommand::ObtainTicketbooks(return_sender, _) => return_no_account(return_sender),
                    // We don't even have an identity at this point, so we might as well not do anything
                    AccountCommand::ResetDeviceIdentity(return_sender, _) => return_sender.send(Ok(())),

                    AccountCommand::RegisterAccount(return_sender, _, _) => return_no_account(return_sender),
                    AccountCommand::RefreshAccountState(return_sender) => return_no_account(return_sender),

                    AccountCommand::VpnApiFirewallDown(return_sender) =>  {
                        shared_state.firewall_active = false;
                        return_sender.send(Ok(()));
                    },
                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        shared_state.firewall_active = true;
                        return_sender.send(Ok(()));
                    },

                    AccountCommand::Common(common_command) => {
                        match common_command {
                            CommonCommand::SetResolverOverrides(return_sender, resolver_overrides) => return_sender.send(common_handler::handle_set_resolver_overrides(shared_state, resolver_overrides)),

                            CommonCommand::GetAccountIdentity(return_sender) => return_sender.send(Ok(None)),
                            CommonCommand::GetStoredAccount(return_sender) => return_sender.send(Ok(None)),
                            CommonCommand::GetDeviceIdentity(return_sender) => return_sender.send(Ok(None)),

                            CommonCommand::GetUsage(return_sender) => return_no_account(return_sender),
                            CommonCommand::GetDevices(return_sender) => return_no_account(return_sender),
                            CommonCommand::GetActiveDevices(return_sender) => return_no_account(return_sender),
                            CommonCommand::GetAvailableTickets(return_sender) => return_no_account(return_sender),
                        }
                    },
                }
                NextAccountControllerState::SameState(self)
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextAccountControllerState::NewState(OfflineState::enter())
                } else {
                    NextAccountControllerState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                NextAccountControllerState::Finished
            }
        }
    }
}

fn return_no_account<S>(result_tx: ReturnSender<S, AccountCommandError>)
where
    S: std::fmt::Debug + std::marker::Send,
{
    result_tx.send(Err(AccountCommandError::NoAccountStored))
}
