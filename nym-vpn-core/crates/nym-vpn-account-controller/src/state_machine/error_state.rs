// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::pin::Pin;

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

pub struct ErrorState {
    refresh_timer: Pin<Box<Sleep>>,
}

impl ErrorState {
    pub fn enter(
        reason: impl ToString,
    ) -> (
        Box<dyn AccountControllerStateHandler>,
        PrivateAccountControllerState,
    ) {
        println!("Entering error state with reason : {}", reason.to_string());
        let refresh_timer = Box::pin(tokio::time::sleep(ACCOUNT_UPDATE_INTERVAL));

        (
            Box::new(Self { refresh_timer }),
            PrivateAccountControllerState::Error,
        )
    }
}

#[async_trait::async_trait]
impl AccountControllerStateHandler for ErrorState {
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
                    AccountCommand::CreateAccount(_) => {}, // SW implement along better error handling
                    AccountCommand::StoreAccount(_, _) => {}, // SW implement along better error handling
                    AccountCommand::RegisterAccount(_, _, _) => {}, // SW implement along better error handling
                    AccountCommand::ForgetAccount(return_sender) => {
                        let res = handler::handle_forget_account(shared_state).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        if error {
                            return NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0));
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

                    AccountCommand::Common(common_command) => {
                        common_handler::handle_common_command(common_command, shared_state).await
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
