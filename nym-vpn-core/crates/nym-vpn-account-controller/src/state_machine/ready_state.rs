// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::pin::Pin;

use nym_vpn_lib_types::{CreateAccountError, RegisterAccountError, StoreAccountError};
use tokio::{sync::mpsc, time::Sleep};
use tokio_util::sync::CancellationToken;

use crate::{
    SharedAccountState,
    commands::{AccountCommand, common_handler, handler},
    state_machine::{
        ACCOUNT_UPDATE_INTERVAL, AccountControllerStateHandler, LoggedOutState,
        NextAccountControllerState, OfflineState, PrivateAccountControllerState, SyncingState,
    },
};

pub struct ReadyState {
    refresh_timer: Pin<Box<Sleep>>,
}

impl ReadyState {
    pub fn enter() -> (
        Box<dyn AccountControllerStateHandler>,
        PrivateAccountControllerState,
    ) {
        let refresh_timer = Box::pin(tokio::time::sleep(ACCOUNT_UPDATE_INTERVAL));

        (
            Box::new(Self { refresh_timer }),
            PrivateAccountControllerState::ReadyToConnect,
        )
    }
}

#[async_trait::async_trait]
impl AccountControllerStateHandler for ReadyState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState,
    ) -> NextAccountControllerState {
        tokio::select! {
            _ = &mut self.refresh_timer => {
                NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0))
            },
            Some(command) = command_rx.recv() => {
                match command {
                    AccountCommand::CreateAccount(return_sender) => {return_sender.send(Err(CreateAccountError::internal("An account already exists")));}, // SW Improve error handling
                    AccountCommand::StoreAccount(return_sender, _) => {return_sender.send(Err(StoreAccountError::internal("An account already exists")));}, // SW Improve error handling
                    AccountCommand::RegisterAccount(return_sender, _, _) => {return_sender.send(Err(RegisterAccountError::internal("An account already exists")));}, // SW Improve error handling
                    AccountCommand::ForgetAccount(return_sender) => {
                        let res = handler::handle_forget_account(shared_state).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        if error {
                            return NextAccountControllerState::SameState(self); // SW we might be in an intermediate state here, double check that
                        } else {
                            return NextAccountControllerState::NewState(LoggedOutState::enter());
                        }
                    },
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        let res = handler::handle_reset_device_identity(shared_state, seed).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        if error {
                            return NextAccountControllerState::SameState(self);
                        } else {
                            return NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0));
                        }
                    },
                    AccountCommand::RefreshAccountState(return_sender) => {
                        return_sender.send(Ok(()));
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0));
                    },

                    AccountCommand::Common(common_command) => common_handler::handle_common_command(common_command, shared_state).await,
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
