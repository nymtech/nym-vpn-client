// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::{
    VpnApiClient,
    error::VpnApiClientError,
    response::{NymErrorResponse, NymVpnAccountStatusResponse},
    types::{Device, VpnApiAccount},
};
use nym_vpn_lib_types::{AccountCommandError, AccountControllerErrorStateReason};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    SharedAccountState,
    commands::{AccountCommand, common_handler, handler},
    state_machine::{
        AccountControllerStateHandler, ErrorState, LoggedOutState, NextAccountControllerState,
        OfflineState, PrivateAccountControllerState,
    },
};
use requesting_zknym_state::RequestingZkNymsState;

mod requesting_zknym_state;

const MAX_SYNCING_ATTEMPTS: u32 = 10;
const SYNCING_STATE_CONTEXT: &str = "SYNCING_STATE";

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
            return ErrorState::enter(
                SyncError::Internal("Logged in, but no device keys".into()).into(),
            );
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
        vpn_api_client: &VpnApiClient,
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
                tracing::debug!("{account_summary_with_device:#?}");

                // Checking that the account is active
                if account_summary_with_device.account_summary.account.status
                    != NymVpnAccountStatusResponse::Active
                {
                    return Err(SyncError::InactiveAccount(
                        account_summary_with_device
                            .account_summary
                            .account
                            .status
                            .to_string(),
                    ));
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
                    != account_summary_with_device
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
                match NymErrorResponse::try_from(e) {
                    Ok(error) => {
                        // SW Use UUID when it will be available
                        if error.status == "access_denied" && error.message == "Account not found" {
                            // Request was fine, but account is unregistered
                            // Later down the line we can maybe register it here
                            Err(SyncError::UnregisteredAccount)
                        } else {
                            Err(SyncError::ApiResponseError {
                                code_reference_id: error.code_reference_id,
                            })
                        }
                    }

                    Err(e) => {
                        tracing::error!("Error trying to get account summary : {e}");
                        Err(SyncError::ApiRequestError)
                    }
                }
            }
        }
    }

    async fn register_device(
        vpn_api_client: &VpnApiClient,
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
                            Ok(fair_usage) => { NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state, self.attempts, fair_usage))},
                            Err(SyncError::ApiRequestError) => {
                                if self.attempts > MAX_SYNCING_ATTEMPTS {
                                    NextAccountControllerState::NewState(ErrorState::enter(SyncError::ApiRequestError.into()))
                                } else {
                                    NextAccountControllerState::NewState(SyncingState::enter(shared_state, self.attempts + 1))
                                }
                            },
                            Err(e) => {NextAccountControllerState::NewState(ErrorState::enter(e.into()))},
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to join on the syncing task : {e}");
                        if self.attempts > MAX_SYNCING_ATTEMPTS {
                            NextAccountControllerState::NewState(ErrorState::enter(SyncError::Internal("Failed to join on the syncing task".into()).into()))
                        } else {
                            NextAccountControllerState::NewState(SyncingState::enter(shared_state, self.attempts + 1))
                        }
                    }
                }
            },
            Some(command) = command_rx.recv() => {
                match command {
                    AccountCommand::CreateAccount(return_sender) => return_sender.send(Err(AccountCommandError::ExistingAccount)),
                    AccountCommand::StoreAccount(return_sender, _) => return_sender.send(Err(AccountCommandError::ExistingAccount)),
                    AccountCommand::RegisterAccount(return_sender, _, _) => return_sender.send(Err(AccountCommandError::ExistingAccount)),
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

#[derive(Debug)]
enum SyncError {
    Internal(String),
    InactiveAccount(String),
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
        match NymErrorResponse::try_from(value) {
            Ok(e) => SyncError::ApiResponseError {
                code_reference_id: e.code_reference_id,
            },
            Err(_) => SyncError::ApiRequestError,
        }
    }
}

impl From<SyncError> for AccountControllerErrorStateReason {
    fn from(value: SyncError) -> Self {
        use SyncError::*;
        match value {
            Internal(details) => Self::Internal {
                context: SYNCING_STATE_CONTEXT.into(),
                details,
            },
            InactiveAccount(status) => Self::AccountStatusNotActive { status },
            UnregisteredAccount => Self::AccountStatusNotActive {
                status: "unregistered".into(),
            },
            InactiveSubscription => Self::InactiveSubscription,
            ApiRequestError => Self::ApiFailure {
                context: SYNCING_STATE_CONTEXT.into(),
                details: "".into(),
            },
            ApiResponseError { code_reference_id } => Self::ApiFailure {
                context: SYNCING_STATE_CONTEXT.into(),
                details: code_reference_id.unwrap_or("No code reference id".into()),
            },
            DeviceTimeDesynced => Self::DeviceTimeDesynced,
            MaxDeviceReached => Self::MaxDeviceReached,
            FairUsageDepleted => Self::BandwidthExceeded {
                context: SYNCING_STATE_CONTEXT.into(),
            },
        }
    }
}
