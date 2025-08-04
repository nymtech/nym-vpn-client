// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::StatusCode;
use nym_vpn_api_client::{
    VpnApiClientError,
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

const MAX_SYNCING_ATTEMPTS: u32 = 10;

pub struct SyncingState {
    syncing_state_handle: JoinHandle<Result<bool, SyncError>>,
    attempts: u32,
}

impl SyncingState {
    pub fn enter(
        shared_state: &SharedAccountState,
        attempts: u32,
    ) -> (
        Box<dyn AccountControllerStateHandler>,
        PrivateAccountControllerState,
    ) {
        let Some(vpn_api_account) = shared_state.vpn_api_account.clone() else {
            return LoggedOutState::enter();
        };
        let Some(device) = shared_state.device.clone() else {
            return ErrorState::enter("Logged in, but no device keys, this shouldn't happen");
        };

        let vpn_api_client = shared_state.vpn_api_client.clone();

        let syncing_state_handle = tokio::spawn(async move {
            SyncingState::syncing_account(&vpn_api_client, &vpn_api_account, &device).await
        });

        (
            Box::new(Self {
                syncing_state_handle,
                attempts,
            }),
            PrivateAccountControllerState::Syncing,
        )
    }

    async fn syncing_account(
        vpn_api_client: &AccountControllerVpnApiClient,
        vpn_api_account: &VpnApiAccount,
        device: &Device,
    ) -> Result<bool, SyncError> {
        // Make sure time isn't too much desynced, othersiwe Zk-nyms will fail to verify on gateways
        if !vpn_api_client
            .get_remote_time()
            .await?
            .is_acceptable_synced()
        {
            return Err(SyncError::DeviceTimeDesynced);
        }

        match vpn_api_client
            .get_account_summary_with_device(vpn_api_account, device)
            .await
        {
            Ok(account_summary_with_device) => {
                println!("{account_summary_with_device:#?}");

                // Checking that the account is active
                if account_summary_with_device.account_summary.account.status
                    != NymVpnAccountStatusResponse::Active
                {
                    return Err(SyncError::InactiveAccount);
                }

                // that there is an active subscription
                if !account_summary_with_device
                    .account_summary
                    .subscription
                    .is_active
                {
                    return Err(SyncError::InactiveSubscription);
                }

                let fair_usage_left = account_summary_with_device
                    .account_summary
                    .fair_usage
                    .limitGB
                    == account_summary_with_device
                        .account_summary
                        .fair_usage
                        .usedGB;

                // that the device is registered or there is a spot left for it with fair usage
                if account_summary_with_device.active_device.is_none() {
                    if account_summary_with_device
                        .account_summary
                        .devices
                        .remaining
                        == 0
                    {
                        return Err(SyncError::MaxDeviceReached); // Early detection of max device reached
                    }

                    // Unregistered device and no fair usage
                    if !fair_usage_left {
                        Err(SyncError::FairUsageDepleted)
                    } else {
                        SyncingState::register_device(vpn_api_client, vpn_api_account, device).await
                    }
                } else {
                    Ok(fair_usage_left)
                }
            }

            Err(e) => {
                match e.get_nym_error_response() {
                    Some(error) => {
                        if e.get_status_code() == Some(StatusCode::FORBIDDEN)
                            && error.message == "Account not found"
                        {
                            // Request was fine, but account is unregistered
                            // SW Use UUID when it will be available
                            SyncingState::register_account(vpn_api_client, vpn_api_account).await
                        } else {
                            Err(SyncError::ApiResponseError {
                                code_reference_id: error.code_reference_id,
                            })
                        }
                    }

                    None => {
                        tracing::error!("Error trying to get account summary : {e}");
                        Err(SyncError::ApiRequestError)
                    }
                }
            }
        }
    }

    async fn register_account(
        _vpn_api_client: &AccountControllerVpnApiClient,
        _vpn_api_account: &VpnApiAccount,
    ) -> Result<bool, SyncError> {
        // Unimplemented for now
        // SW Do we want to register account automatically here or not?
        Err(SyncError::UnregisteredAccount)
    }

    async fn register_device(
        vpn_api_client: &AccountControllerVpnApiClient,
        vpn_api_account: &VpnApiAccount,
        device: &Device,
    ) -> Result<bool, SyncError> {
        vpn_api_client
            .register_device(vpn_api_account, device)
            .await?;
        Ok(true) // We can register a device, we have fair usage
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
                            Ok(fair_usage) => { NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state, self.attempts, fair_usage))},
                            Err(SyncError::UnregisteredAccount) => {NextAccountControllerState::NewState(ErrorState::enter("no account"))},
                            Err(SyncError::InactiveAccount) => {NextAccountControllerState::NewState(ErrorState::enter("inactive account"))},
                            Err(SyncError::InactiveSubscription) => {NextAccountControllerState::NewState(ErrorState::enter("inactive sub"))},
                            Err(SyncError::MaxDeviceReached)=> {NextAccountControllerState::NewState(ErrorState::enter("max device reached"))},
                            Err(SyncError::ApiRequestError) => {
                                if self.attempts > MAX_SYNCING_ATTEMPTS {
                                    NextAccountControllerState::NewState(ErrorState::enter("Api failure : Max attempts reached"))
                                } else {
                                    NextAccountControllerState::NewState(SyncingState::enter(shared_state, self.attempts + 1))
                                }
                            },
                            Err(SyncError::ApiResponseError { code_reference_id }) => {NextAccountControllerState::NewState(ErrorState::enter(format!("api error : {code_reference_id:?}")))},
                            Err(SyncError::FairUsageDepleted) => {NextAccountControllerState::NewState(ErrorState::enter("fair usage depleted"))},
                            Err(SyncError::DeviceTimeDesynced) => {NextAccountControllerState::NewState(ErrorState::enter("Device time desynced"))}
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to join on the syncing task : {e}");
                        NextAccountControllerState::NewState(SyncingState::enter(shared_state, self.attempts + 1))
                    }
                }
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
                            return NextAccountControllerState::SameState(self);
                        } else {
                            self.syncing_state_handle.abort();
                            return NextAccountControllerState::NewState(LoggedOutState::enter());
                        }
                    },
                    AccountCommand::RefreshAccountState(return_sender) => {
                        self.syncing_state_handle.abort();
                        return_sender.send(Ok(()));
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state,0));
                    },
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                        self.syncing_state_handle.abort();
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state,0));
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

// SW Add conversion from this to Error Reason
#[derive(Debug)]
enum SyncError {
    InactiveAccount,
    UnregisteredAccount,
    InactiveSubscription,
    ApiRequestError,
    ApiResponseError { code_reference_id: Option<String> },
    DeviceTimeDesynced,
    MaxDeviceReached,
    FairUsageDepleted,
}

impl From<VpnApiClientError> for SyncError {
    fn from(value: VpnApiClientError) -> Self {
        match value.get_nym_error_response() {
            Some(e) => SyncError::ApiResponseError {
                code_reference_id: e.code_reference_id,
            },
            None => SyncError::ApiRequestError,
        }
    }
}
