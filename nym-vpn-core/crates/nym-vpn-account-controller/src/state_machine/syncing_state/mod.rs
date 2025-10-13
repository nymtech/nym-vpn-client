// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    error::VpnApiClientError,
    response::{NymErrorResponse, NymVpnAccountStatusResponse},
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::{AccountCommandError, AccountControllerErrorStateReason};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    SharedAccountState,
    commands::{AccountCommand, common_handler, handler},
    state_machine::{
        AccountControllerStateHandler, ErrorState, LoggedOutState, NextAccountControllerState,
        OfflineState, PrivateAccountControllerState, decentralised_state::DecentralisedState,
    },
};
use requesting_zknym_state::RequestingZkNymsState;

mod requesting_zknym_state;

const MAX_SYNCING_ATTEMPTS: u32 = 10;
const SYNCING_STATE_CONTEXT: &str = "SYNCING_STATE";

/// Syncing State
/// This is the heart of the Account Controller.
/// That state has to determine where we are at :
/// - Is an account stored?
/// - Is the stored account registered ?
/// - Is the subscription active ?
/// - Is the current device registered ?
/// - Do we have fair usage left ?
///
/// A retry mechanism is in place, if the error is in the API requests. Other errors lead to the ErrorState since they are not recoverable.  
///
/// Possible next state :
/// - LoggedOutState : No account is stored
/// - RequestingZkNymState : Everything is fine on the account front, we just to check on our ZK-nyms storage before being ready to connect
/// - SyncingState : We try again if there was an error while making an API request
/// - ErrorState : An actual error happened, or one of the above questions has a negative answers, preventing us to proceed.
/// - OfflineState : the connectivity monitor is telling we're not connected
/// - DecentralisedState : The loaded account is set to "decentralised" mode
pub struct SyncingState {
    syncing_state_handle: JoinHandle<Result<bool, SyncError>>,
    attempts: u32,
}

impl SyncingState {
    pub fn enter<C: ConnectivityMonitor>(
        shared_state: &SharedAccountState<C>,
        attempts: u32,
    ) -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        let Some(vpn_api_account) = shared_state.vpn_api_account.clone() else {
            return LoggedOutState::enter();
        };
        if vpn_api_account.mode().is_decentralised() {
            return DecentralisedState::enter();
        }
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
        vpn_api_account: &VpnAccount,
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
            Ok(summary) => {
                tracing::debug!("{summary:#?}");

                // Checking that the account is active
                if !summary.account_active() {
                    return Err(SyncError::InactiveAccount(
                        summary.account_summary.account.status.to_string(),
                    ));
                }

                // that there is an active subscription
                if !summary.subscription_active() {
                    return Err(SyncError::InactiveSubscription);
                }

                let fair_usage_left = summary.bandwidth_limit() != summary.used_bandwidth();

                // that the device is registered or there is a spot left for it with fair usage
                if summary.active_device.is_none() {
                    if summary.remaining_devices() == 0 {
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
                let error_response = NymErrorResponse::try_from(e)?;
                // SW Use UUID when it will be available
                if error_response.status == "access_denied"
                    && error_response.message == "Account not found"
                {
                    // Request was fine, but account is unregistered
                    // Later down the line we can maybe register it here
                    Err(SyncError::UnregisteredAccount)
                } else {
                    Err(SyncError::ApiResponseError {
                        details: error_response
                            .code_reference_id
                            .unwrap_or(error_response.message),
                    })
                }
            }
        }
    }

    async fn register_device(
        vpn_api_client: &VpnApiClient,
        vpn_api_account: &VpnAccount,
        device: &Device,
    ) -> Result<bool, SyncError> {
        vpn_api_client
            .register_device(vpn_api_account, device)
            .await?;
        Ok(true) // We can register a device, we have fair usage
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for SyncingState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C> {
        tokio::select! {
            biased;
            _ = shutdown_token.cancelled() => {
                self.syncing_state_handle.abort();
                NextAccountControllerState::Finished
            }
            syncing_result = &mut self.syncing_state_handle => {
                match syncing_result {
                    Ok(result) => {
                        match result {
                            Ok(fair_usage) => { NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state, self.attempts, fair_usage))},
                            Err(e) if e.is_retryable() => {
                                if self.attempts > MAX_SYNCING_ATTEMPTS {
                                    tracing::debug!("Error trying to get account summary, exhausted retries : {}", e.to_string());
                                    NextAccountControllerState::NewState(ErrorState::enter(e.into()))
                                } else {
                                    tracing::debug!("Error trying to get account summary, retrying : {}", e.to_string());
                                    NextAccountControllerState::NewState(SyncingState::enter(shared_state, self.attempts + 1))
                                }
                            },
                            Err(e) => {
                                tracing::debug!("Error trying to get account summary, not retrying : {}", e.to_string());
                                NextAccountControllerState::NewState(ErrorState::enter(e.into()))
                            },
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
                    AccountCommand::RegisterAccount(return_sender, account, platform) => {
                        let res = handler::handle_register_account(shared_state, account, platform).await;
                        return_sender.send(res);
                    }
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
                    AccountCommand::RotateKeys(return_sender) => {
                        let res = handler::handle_rotate_keys(shared_state).await;
                        return_sender.send(res);
                    },
                    AccountCommand::AccountBalance(return_sender) => return_sender.send(Err(AccountCommandError::AccountNotDecentralised)),
                    AccountCommand::ObtainTicketbooks(return_sender, _) => return_sender.send(Err(AccountCommandError::AccountNotDecentralised)),
                    AccountCommand::RefreshAccountState(return_sender) => {
                        return_sender.send(Ok(()));
                        if shared_state.firewall_active {
                            return NextAccountControllerState::SameState(self);
                        } else {
                            self.syncing_state_handle.abort();
                            return NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0));
                        }
                    },
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                        self.syncing_state_handle.abort();
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state,0));
                    },

                    AccountCommand::VpnApiFirewallDown(return_sender) =>  {
                        shared_state.firewall_active = false;
                        return_sender.send(Ok(()));
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state, self.attempts));
                    },

                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        shared_state.firewall_active = true;
                        self.syncing_state_handle.abort();
                        return_sender.send(Ok(()));
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
        }
    }
}

#[derive(Debug, strum::Display)]
enum SyncError {
    Internal(String),
    InactiveAccount(String),
    UnregisteredAccount,
    InactiveSubscription,
    ApiRequestError(String),
    ApiResponseError { details: String },
    DeviceTimeDesynced,
    MaxDeviceReached,
    FairUsageDepleted,
}

impl SyncError {
    fn is_retryable(&self) -> bool {
        matches!(
            self,
            SyncError::ApiRequestError(_)
                | SyncError::DeviceTimeDesynced
                | SyncError::InactiveSubscription // in the case of IAP, it might take a while for the subscription to become active
        )
    }
}

impl From<VpnApiClientError> for SyncError {
    fn from(value: VpnApiClientError) -> Self {
        match NymErrorResponse::try_from(value) {
            Ok(error_response) => SyncError::ApiResponseError {
                details: error_response
                    .code_reference_id
                    .unwrap_or(error_response.message),
            },
            Err(e) => SyncError::ApiRequestError(e.to_string()),
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
            ApiRequestError(e) => Self::ApiFailure {
                context: SYNCING_STATE_CONTEXT.into(),
                details: e,
            },
            ApiResponseError { details } => Self::ApiFailure {
                context: SYNCING_STATE_CONTEXT.into(),
                details,
            },
            DeviceTimeDesynced => Self::DeviceTimeDesynced,
            MaxDeviceReached => Self::MaxDeviceReached,
            FairUsageDepleted => Self::BandwidthExceeded {
                context: SYNCING_STATE_CONTEXT.into(),
            },
        }
    }
}
