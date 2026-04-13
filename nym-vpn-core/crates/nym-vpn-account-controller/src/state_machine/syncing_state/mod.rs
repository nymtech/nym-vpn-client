// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{cmp::min, sync::Arc, time::Duration};

use crate::{
    SharedAccountState,
    commands::{AccountCommand, UpgradeModeCommand, common_handler, handler},
    state_machine::{
        AccountControllerStateHandler, DecentralisedState, ErrorState, LoggedOutState,
        NextAccountControllerState, OfflineState, PendingSubscriptionState,
        PrivateAccountControllerState,
    },
};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    error::VpnApiClientError,
    response::{NymErrorResponse, NymVpnAccountSummaryWithDeviceResponse},
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerErrorStateReason, VpnAccountSummary,
};
use requesting_zknym_state::RequestingZkNymsState;
use tokio::sync::mpsc;
use tokio_util::sync::{CancellationToken, DropGuard};
use tracing::warn;

pub(super) mod requesting_zknym_state;

const MAX_SYNCING_ATTEMPTS: u32 = 10;
const SYNCING_STATE_CONTEXT: &str = "SYNCING_STATE";

// bounded exponential backoff for retries [0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0] = 55.75s max wait
const RETRY_BACKOFF: Duration = Duration::from_millis(250);
const MAX_BACKOFF_EXPONENT: u32 = 5;
const BACKOFF_BASE: u32 = 2;

enum SyncEvent {
    /// Account summary is received
    AccountSummary(Box<VpnAccountSummary>),

    /// Failure to complete synchronization
    Failure(SyncError),

    /// Finished synchronization without errors
    Finished,
}

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
    attempts: u32,
    event_rx: mpsc::UnboundedReceiver<SyncEvent>,
    sync_cancel_token: Option<DropGuard>,
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
            return ErrorState::enter(AccountControllerErrorStateReason::Internal {
                context: SYNCING_STATE_CONTEXT.into(),
                details: "Logged in, but no device keys".into(),
            });
        };

        let vpn_api_client = shared_state.vpn_api_client.clone();

        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let sync_cancel_token = CancellationToken::new();

        // This handle does not need to be awaited since event channel and cancellation token are sufficient.
        let _syncing_state_handle =
            tokio::spawn(sync_cancel_token.child_token().run_until_cancelled_owned(
                SyncingState::sync_account(
                    event_tx,
                    vpn_api_client,
                    vpn_api_account,
                    device,
                    attempts,
                ),
            ));

        (
            Box::new(Self {
                attempts,
                event_rx,
                sync_cancel_token: Some(sync_cancel_token.drop_guard()),
            }),
            PrivateAccountControllerState::Syncing,
        )
    }

    async fn sync_account(
        event_tx: mpsc::UnboundedSender<SyncEvent>,
        vpn_api_client: VpnApiClient,
        vpn_api_account: Arc<VpnAccount>,
        device: Device,
        attempts: u32,
    ) {
        if attempts > 0 {
            tokio::time::sleep(Self::get_delay(attempts)).await;
        }

        let final_event =
            Self::sync_account_inner(event_tx.clone(), vpn_api_client, vpn_api_account, device)
                .await
                .map(|_| SyncEvent::Finished)
                .unwrap_or_else(SyncEvent::Failure);
        event_tx.send(final_event).ok();
    }

    async fn sync_account_inner(
        event_tx: mpsc::UnboundedSender<SyncEvent>,
        vpn_api_client: VpnApiClient,
        vpn_api_account: Arc<VpnAccount>,
        device: Device,
    ) -> Result<(), SyncError> {
        // Make sure time isn't too much desynced, othersiwe Zk-nyms will fail to verify on gateways
        let remote_time = vpn_api_client
            .get_remote_time()
            .await
            .map_err(Self::map_vpn_api_error)?;

        if remote_time.is_acceptable_synced() {
            let summary = vpn_api_client
                .get_account_summary_with_device(&vpn_api_account, &device)
                .await
                .map_err(Self::map_vpn_api_error)?;

            tracing::debug!("{summary:#?}");

            Self::handle_received_account_summary(
                event_tx.clone(),
                vpn_api_client,
                &vpn_api_account,
                &device,
                summary,
            )
            .await
        } else {
            Err(SyncError::DeviceTimeDesynced)
        }
    }

    async fn handle_received_account_summary(
        event_tx: mpsc::UnboundedSender<SyncEvent>,
        vpn_api_client: VpnApiClient,
        vpn_api_account: &VpnAccount,
        device: &Device,
        summary: NymVpnAccountSummaryWithDeviceResponse,
    ) -> Result<(), SyncError> {
        let mut vpn_account_summary = VpnAccountSummary::try_from(&summary.account_summary)
            .map_err(|err| SyncError::ApiResponseError {
                details: format!("Failed to create account summary from API response: {err}"),
            })?;

        // todo: refactor, this should not be here.
        vpn_account_summary.account_mode =
            Some(nym_vpn_store::types::StoredAccountMode::from(vpn_api_account.mode()).into());

        // Propagate account summary even if sync eventually fails.
        let _ = event_tx.send(SyncEvent::AccountSummary(Box::new(
            vpn_account_summary.clone(),
        )));

        // Checking that the account is active
        if !summary.account_active() {
            Err(SyncError::InactiveAccount(
                summary.account_summary.account.status.to_string(),
            ))
        } else if summary.subscription_pending() {
            // subscription exists but is not yet active (e.g. cash payment still processing)
            Err(SyncError::PendingSubscription)
        } else if !summary.subscription_active() {
            // that there is an active subscription
            Err(SyncError::InactiveSubscription)
        } else if summary.active_device.is_none() {
            // that the device is registered or there is a spot left for it with fair usage
            if summary.remaining_devices() == 0 {
                Err(SyncError::MaxDeviceReached) // Early detection of max device reached
            } else if !vpn_account_summary.fair_usage_left() {
                Err(SyncError::FairUsageDepleted)
            } else {
                SyncingState::register_device(&vpn_api_client, vpn_api_account, device).await
            }
        } else {
            Ok(())
        }
    }

    fn map_vpn_api_error(err: VpnApiClientError) -> SyncError {
        match NymErrorResponse::try_from(err) {
            Ok(error_response) => {
                // SW Use UUID when it will be available
                if error_response.status == "access_denied"
                    && error_response.message == "Account not found"
                {
                    // Request was fine, but account is unregistered
                    // Later down the line we can maybe register it here
                    SyncError::UnregisteredAccount
                } else {
                    SyncError::ApiResponseError {
                        details: error_response
                            .code_reference_id
                            .unwrap_or(error_response.message),
                    }
                }
            }
            Err(err) => SyncError::from(err),
        }
    }

    async fn register_device(
        vpn_api_client: &VpnApiClient,
        vpn_api_account: &VpnAccount,
        device: &Device,
    ) -> Result<(), SyncError> {
        vpn_api_client
            .register_device(vpn_api_account, device)
            .await?;
        Ok(()) // We can register a device, we have fair usage
    }

    /// The attempt retries should  start with attempt 1
    fn get_delay(attempts: u32) -> Duration {
        RETRY_BACKOFF * BACKOFF_BASE.pow(min(attempts - 1, MAX_BACKOFF_EXPONENT))
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
                NextAccountControllerState::Finished
            }
            Some(sync_event) = self.event_rx.recv() => {
                match sync_event {
                    SyncEvent::AccountSummary(vpn_account_summary) => {
                        shared_state.vpn_account_summary = Some(*vpn_account_summary);
                        NextAccountControllerState::SameState(self)
                    }
                    SyncEvent::Failure(err) => {
                        let is_retryable = err.is_retryable();
                        let err_str = err.to_string();
                        match err.into_error_reason() {
                            None => {
                                tracing::debug!("Subscription is pending, waiting before retrying");
                                NextAccountControllerState::NewState(PendingSubscriptionState::enter())
                            }
                            Some(reason) if is_retryable => {
                                if self.attempts > MAX_SYNCING_ATTEMPTS {
                                    tracing::debug!("Error trying to get account summary, exhausted retries : {err_str}");
                                    NextAccountControllerState::NewState(ErrorState::enter(reason))
                                } else {
                                    tracing::debug!(
                                        "Error trying to get account summary attempt {}, retrying after {:?} : {err_str}",
                                        self.attempts,
                                        Self::get_delay(self.attempts + 1),
                                    );
                                    NextAccountControllerState::NewState(SyncingState::enter(shared_state, self.attempts + 1))
                                }
                            }
                            Some(reason) => {
                                tracing::debug!("Error trying to get account summary, not retrying : {err_str}");
                                NextAccountControllerState::NewState(ErrorState::enter(reason))
                            }
                        }
                    }
                    SyncEvent::Finished => {
                        NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state, self.attempts, false))
                    }
                }
            }
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
                        return if error {
                            NextAccountControllerState::SameState(self)
                        } else {
                            NextAccountControllerState::NewState(LoggedOutState::enter())
                        }
                    },
                    AccountCommand::LinkAccount(return_sender, privy_account) => {
                        let res = handler::handle_link_account(shared_state, privy_account).await;
                        return_sender.send(res);
                    },
                    AccountCommand::RotateKeys(return_sender) => {
                        let res = handler::handle_rotate_keys(shared_state).await;
                        return_sender.send(res);
                    },
                    AccountCommand::AccountBalance(return_sender) => return_sender.send(Err(AccountCommandError::AccountNotDecentralised)),
                    AccountCommand::ObtainTicketbooks(return_sender, _) => return_sender.send(Err(AccountCommandError::AccountNotDecentralised)),
                    AccountCommand::RefreshAccountState(return_sender) => {
                        return_sender.send(Ok(()));
                        return if shared_state.firewall_active {
                            NextAccountControllerState::SameState(self)
                        } else {
                            NextAccountControllerState::NewState(SyncingState::enter(shared_state, 0))
                        }
                    },
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                        return NextAccountControllerState::NewState(SyncingState::enter(shared_state,0));
                    },

                    AccountCommand::VpnApiFirewallDown(return_sender) =>  {
                        return_sender.send(Ok(()));
                        // No-op if the firewall was already down
                        if shared_state.firewall_active {
                            shared_state.firewall_active = false;
                            return NextAccountControllerState::NewState(SyncingState::enter(shared_state, self.attempts));
                        }
                    },

                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        shared_state.firewall_active = true;
                        // Explicitly cancel sync task since the same state persists.
                        // Sync will restart once firewall permits traffic to flow again.
                        self.sync_cancel_token.take();
                        return_sender.send(Ok(()));
                    },

                    AccountCommand::Common(common_command) => {
                        common_handler::handle_common_command(common_command, shared_state).await
                    },
                    AccountCommand::UpgradeMode(upgrade_mode_command) => match upgrade_mode_command {
                        UpgradeModeCommand::GetUpgradeModeEnabled(return_sender) => {
                            return_sender.send(Ok(false))
                        }
                        UpgradeModeCommand::DisableUpgradeMode(return_sender) => {
                            warn!(
                                "received unexpected command to disable upgrade mode while in 'SyncingState' state"
                            );
                            return_sender.send(Ok(()))
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
        }
    }
}

#[derive(Debug, strum::Display)]
enum SyncError {
    InactiveAccount(String),
    UnregisteredAccount,
    InactiveSubscription,
    PendingSubscription,
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

    /// Returns the corresponding error reason for the error state, or `None` if the error
    /// should not result in an error state (e.g. pending subscription has its own state).
    fn into_error_reason(self) -> Option<AccountControllerErrorStateReason> {
        use SyncError::*;
        match self {
            PendingSubscription => None,
            InactiveAccount(status) => {
                Some(AccountControllerErrorStateReason::AccountStatusNotActive { status })
            }
            UnregisteredAccount => {
                Some(AccountControllerErrorStateReason::AccountStatusNotActive {
                    status: "unregistered".into(),
                })
            }
            InactiveSubscription => Some(AccountControllerErrorStateReason::InactiveSubscription),
            ApiRequestError(e) => Some(AccountControllerErrorStateReason::ApiFailure {
                context: SYNCING_STATE_CONTEXT.into(),
                details: e,
            }),
            ApiResponseError { details } => Some(AccountControllerErrorStateReason::ApiFailure {
                context: SYNCING_STATE_CONTEXT.into(),
                details,
            }),
            DeviceTimeDesynced => Some(AccountControllerErrorStateReason::DeviceTimeDesynced),
            MaxDeviceReached => Some(AccountControllerErrorStateReason::MaxDeviceReached),
            FairUsageDepleted => Some(AccountControllerErrorStateReason::BandwidthExceeded {
                context: SYNCING_STATE_CONTEXT.into(),
            }),
        }
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
