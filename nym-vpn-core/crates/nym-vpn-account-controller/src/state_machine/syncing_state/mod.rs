// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::StatusCode;
use nym_vpn_api_client::{
    HttpClientError, VpnApiClientError,
    response::NymVpnAccountStatusResponse,
    types::{Device, VpnApiAccount},
};
use nym_vpn_lib_types::{CreateAccountError, RegisterAccountError, StoreAccountError};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    SharedAccountState,
    commands::{AccountCommand, common_handler, handler},
    state_machine::{
        AccountControllerStateHandler, ErrorState, LoggedOutState, NextAccountControllerState,
        OfflineState, PrivateAccountControllerState,
    },
    vpn_api_client::AccountControllerVpnApiClient,
};
use requesting_zknym_state::RequestingZkNymsState;

mod requesting_zknym_state;

pub struct SyncingState {
    syncing_state_handle: JoinHandle<Result<(), SyncError>>,
}

impl SyncingState {
    pub fn enter(
        shared_state: &SharedAccountState,
    ) -> (
        Box<dyn AccountControllerStateHandler>,
        PrivateAccountControllerState,
    ) {
        if shared_state.vpn_api_account.is_none() {
            return LoggedOutState::enter();
        }
        // SW Check time sync before and retries

        //SW Handle these unwraps properly
        #[allow(clippy::unwrap_used)]
        let vpn_api_account = shared_state.vpn_api_account.clone().unwrap();
        #[allow(clippy::unwrap_used)]
        let device = shared_state.device.clone().unwrap();
        let vpn_api_client = shared_state.vpn_api_client.clone();
        let syncing_state_handle = tokio::spawn(async move {
            SyncingState::syncing_account(&vpn_api_client, &vpn_api_account, &device).await
        });

        (
            Box::new(Self {
                syncing_state_handle,
            }),
            PrivateAccountControllerState::Syncing,
        )
    }

    async fn syncing_account(
        vpn_api_client: &AccountControllerVpnApiClient,
        vpn_api_account: &VpnApiAccount,
        device: &Device,
    ) -> Result<(), SyncError> {
        match vpn_api_client
            .get_account_summary_with_device(vpn_api_account, device)
            .await
        {
            // SW Do we handle fair usage here?
            Ok(account_summary_with_device) => {
                if account_summary_with_device.account_summary.account.status
                    != NymVpnAccountStatusResponse::Active
                {
                    Err(SyncError::InactiveAccount)
                } else if !account_summary_with_device
                    .account_summary
                    .subscription
                    .is_active
                {
                    Err(SyncError::InactiveSubscription)
                } else if account_summary_with_device.active_device.is_none() {
                    SyncingState::register_device(vpn_api_client, vpn_api_account, device).await
                } else {
                    Ok(())
                }
            }

            Err(VpnApiClientError::GetAccountSummaryWithDevice(e)) => match e {
                HttpClientError::EndpointFailure { status, error }
                    if status == StatusCode::FORBIDDEN && error.message == "Account not found" =>
                {
                    // SW Use UUID when it will be available
                    SyncingState::register_account(vpn_api_client, vpn_api_account).await
                }

                _ => {
                    tracing::error!("Internal error. This should never happen : {e}");
                    Err(SyncError::ApiFailure)
                }
            },
            Err(e) => {
                tracing::error!(
                    "Received an incorrect error type while sycning account. This should never happen : {e}"
                );
                Err(SyncError::ApiFailure)
            }
        }
    }

    async fn register_account(
        _vpn_api_client: &AccountControllerVpnApiClient,
        _vpn_api_account: &VpnApiAccount,
    ) -> Result<(), SyncError> {
        // Unimplemented for now
        Err(SyncError::UnregisteredAccount)
    }

    async fn register_device(
        vpn_api_client: &AccountControllerVpnApiClient,
        vpn_api_account: &VpnApiAccount,
        device: &Device,
    ) -> Result<(), SyncError> {
        vpn_api_client
            .register_device(vpn_api_account, device)
            .await
            .map_err(|_| SyncError::ApiFailure)?; // SW properly handle error like "too many devices"
        Ok(())
    }
}

#[async_trait::async_trait]
impl AccountControllerStateHandler for SyncingState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState,
    ) -> NextAccountControllerState {
        tokio::select! {
            syncing_result = &mut self.syncing_state_handle => {
                match syncing_result {
                    Ok(result) => {
                        match result {
                            // SW better error handling
                            Ok(()) => { NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state).await)},
                            Err(SyncError::UnregisteredAccount) => {NextAccountControllerState::NewState(ErrorState::enter("no account"))},
                            Err(SyncError::InactiveAccount) => {NextAccountControllerState::NewState(ErrorState::enter("inactive account"))},
                            Err(SyncError::InactiveSubscription) => {NextAccountControllerState::NewState(ErrorState::enter("inactive sub"))},
                            Err(SyncError::ApiFailure) => {NextAccountControllerState::NewState(ErrorState::enter("api error"))},
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to join on the syncing task : {e}");
                        NextAccountControllerState::NewState(SyncingState::enter(shared_state))
                    }
                }
            },
            Some(command) = command_rx.recv() => {
                match command {
                    AccountCommand::CreateAccount(return_sender) => {return_sender.send(Err(CreateAccountError::internal("An account already exists")));}, //SW Improve error handling
                    AccountCommand::StoreAccount(return_sender, _) => {return_sender.send(Err(StoreAccountError::internal("An account already exists")));}, //SW Improve error handling
                    AccountCommand::RegisterAccount(return_sender, _, _) => {return_sender.send(Err(RegisterAccountError::internal("An account already exists")));}, //SW Improve error handling
                    AccountCommand::ForgetAccount(return_sender) => {
                        let res = handler::handle_forget_account(shared_state).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        if error {
                            return NextAccountControllerState::SameState(self); // SW we might be in an intermediate state here, double check that
                        } else {
                            self.syncing_state_handle.abort();
                            return NextAccountControllerState::NewState(LoggedOutState::enter());
                        }
                    },
                    AccountCommand::RefreshAccountState(return_sender) => {
                        self.syncing_state_handle.abort();
                        return_sender.send(Ok(()));
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state));
                    },
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                        self.syncing_state_handle.abort();
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state));
                    },
                    AccountCommand::Common(common_command) => {
                        common_handler::handle_common_command(common_command, shared_state).await
                    },
                }
                NextAccountControllerState::SameState(self)
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    self.syncing_state_handle.abort();
                    NextAccountControllerState::NewState(OfflineState::enter())
                } else {
                    NextAccountControllerState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                self.syncing_state_handle.abort();
                NextAccountControllerState::Finished
            }
        }
    }
}

#[derive(Debug)]
enum SyncError {
    InactiveAccount,
    UnregisteredAccount,
    InactiveSubscription,
    ApiFailure,
}
