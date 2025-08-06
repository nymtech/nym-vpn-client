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
        AccountControllerStateHandler, NextAccountControllerState, PrivateAccountControllerState,
        SyncingState,
    },
};

pub struct OfflineState;

impl OfflineState {
    pub fn enter<C: ConnectivityMonitor>() -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        (Box::new(Self), PrivateAccountControllerState::Offline)
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for OfflineState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C> {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                // Intentionnally no command grouping for clarity
                match command {
                    AccountCommand::CreateAccount(return_sender) => {
                        return_sender.send(handler::handle_create_account(shared_state).await)
                    },
                    AccountCommand::StoreAccount(return_sender, mnemonic) => {
                        return_sender.send(handler::handle_store_account(shared_state, mnemonic).await)
                    },
                    AccountCommand::RegisterAccount(return_sender, _, _) => return_no_connectivity(return_sender),
                    AccountCommand::ForgetAccount(return_sender) => return_no_connectivity(return_sender), // this shouldn't happen, as tunnel state is checked before sending the command
                    // While we can technically do that in offline mode, if we were planning on reconnecting after, trouble ensue
                    AccountCommand::ResetDeviceIdentity(return_sender, _) => return_no_connectivity(return_sender), // this shouldn't happen, as tunnel state is checked before sending the command
                    AccountCommand::RefreshAccountState(return_sender) => return_no_connectivity(return_sender),


                    AccountCommand::Common(common_command) => {
                        match common_command {
                            CommonCommand::GetStoredMnemonic(return_sender) => return_sender.send(common_handler::handle_get_stored_mnemonic(shared_state).await),
                            CommonCommand::GetDeviceIdentity(return_sender) => return_sender.send(common_handler::handle_get_device_identity(shared_state)),
                            CommonCommand::GetAccountIdentity(return_sender) => return_sender.send(common_handler::handle_get_account_identity(shared_state)),
                            CommonCommand::SetStaticApiAddresses(return_sender,socket_addrs) => return_sender.send(common_handler::handle_set_static_api_addresses(shared_state,socket_addrs)),
                            CommonCommand::GetUsage(return_sender) => return_no_connectivity(return_sender),
                            CommonCommand::GetDevices(return_sender) => return_no_connectivity(return_sender),
                            CommonCommand::GetActiveDevices(return_sender) => return_no_connectivity(return_sender),
                            CommonCommand::GetAvailableTickets(return_sender) => return_no_connectivity(return_sender),
                        }

                    },

                }
                NextAccountControllerState::SameState(self)
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextAccountControllerState::SameState(self)
                } else {
                    NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0))
                }
            }
            _ = shutdown_token.cancelled() => {
                NextAccountControllerState::Finished
            }
        }
    }
}

fn return_no_connectivity<S>(result_tx: ReturnSender<S, AccountCommandError>)
where
    S: std::fmt::Debug + std::marker::Send,
{
    result_tx.send(Err(AccountCommandError::Offline))
}
