// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

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

pub struct LoggedOutState;

impl LoggedOutState {
    pub fn enter() -> (
        Box<dyn AccountControllerStateHandler>,
        PrivateAccountControllerState,
    ) {
        (Box::new(Self), PrivateAccountControllerState::LoggedOut)
    }
}

#[async_trait::async_trait]
impl AccountControllerStateHandler for LoggedOutState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState,
    ) -> NextAccountControllerState {
        tokio::select! {
        Some(command) = command_rx.recv() => {
                // Intentionnally no command grouping for clarity
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
                    AccountCommand::ResetDeviceIdentity(return_sender, _) => return_sender.send(Ok(())),

                    AccountCommand::RegisterAccount(return_sender, _, _) => return_no_account(return_sender),
                    AccountCommand::RefreshAccountState(return_sender) => return_no_account(return_sender),

                    AccountCommand::Common(common_command) => {
                        match common_command {
                            CommonCommand::SetStaticApiAddresses(return_sender, socket_addrs) => return_sender.send(common_handler::handle_set_static_api_addresses(shared_state,socket_addrs)),
                            CommonCommand::GetAccountIdentity(return_sender) => return_sender.send(Ok(None)),

                            CommonCommand::GetUsage(return_sender) => return_no_account(return_sender),
                            CommonCommand::GetStoredMnemonic(return_sender) => return_no_account(return_sender),
                            CommonCommand::GetDeviceIdentity(return_sender) => return_no_account(return_sender),
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
