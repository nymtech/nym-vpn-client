// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use crate::{
    SharedAccountState,
    commands::{AccountCommand, common_handler, handler},
    state_machine::{
        AccountControllerStateHandler, DecentralisedState, ErrorState, LoggedOutState,
        NextAccountControllerState, OfflineState, PendingSubscriptionState,
        PrivateAccountControllerState, ReadyState,
        syncing_state::{SyncMode, SyncingNetworkState},
    },
};
use futures::{FutureExt, future::Fuse};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerErrorStateReason, VpnAccountSummary,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::{CancellationToken, DropGuard};

const SYNCING_LOCAL_STATE_CONTEXT: &str = "SYNCING_LOCAL_STATE";

/// How old a cached summary may be before it is considered stale and force-refreshed.
const SUMMARY_STALE_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

/// SyncingLocal State
/// This is the "local" half of syncing: it runs the account checks against the locally cached
/// [`VpnAccountSummary`] without hitting the VPN API, answering:
/// - Is the stored account active ?
/// - Is the subscription active (or pending) ?
/// - Is the current device registered ?
/// - Do we have fair usage left ?
///
/// No other state than SyncingNetworkState should lead there
///
/// If no summary is cached (or it is stale), there is nothing to trust locally, so we defer to
/// [`SyncingNetworkState`] to fetch one first. The only network interaction here is registering the
/// device, which happens as a direct consequence of the checks (a free slot is available).
///
/// A failed check goes straight to the error state (or `PendingSubscriptionState` for a pending
/// subscription); there is no retry, since re-checking the same cached data would not change the
/// outcome - obtaining fresh data is what `SyncingNetworkState` is for.
///
/// Possible next state :
/// - LoggedOutState : No account is stored
/// - SyncingNetworkState : No summary is cached, or the cache is stale and must be refreshed
/// - ReadyState : Everything checks out.
/// - PendingSubscriptionState : The subscription exists but is not yet active
/// - ErrorState : One of the checks failed in a non-recoverable way
/// - OfflineState : the connectivity monitor is telling we're not connected
/// - DecentralisedState : The loaded account is set to "decentralised" mode
pub(crate) struct SyncingLocalState {
    result_rx: Fuse<oneshot::Receiver<Result<bool, SyncError>>>,
    sync_cancel_token: Option<DropGuard>,
}

impl SyncingLocalState {
    pub(crate) async fn enter<C: ConnectivityMonitor>(
        shared_state: &mut SharedAccountState<C>,
    ) -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        let Some(vpn_api_account) = shared_state.vpn_api_account.clone() else {
            return LoggedOutState::enter(shared_state).await;
        };
        if vpn_api_account.mode().is_decentralised() {
            return DecentralisedState::enter();
        }
        let Some(device) = shared_state.device.clone() else {
            return ErrorState::enter(
                shared_state,
                AccountControllerErrorStateReason::Internal {
                    context: SYNCING_LOCAL_STATE_CONTEXT.into(),
                    details: "Logged in, but no device keys".into(),
                },
            )
            .await;
        };

        // Nothing to check locally without a cached summary: go fetch one from the network.
        let Some(summary) = shared_state.vpn_account_summary.clone() else {
            return SyncingNetworkState::enter(shared_state, SyncMode::Mandatory).await;
        };

        // A stale cache can't be trusted; force a mandatory re-fetch instead of checking it.
        if summary.is_stale(SUMMARY_STALE_AFTER) {
            tracing::debug!("Cached account summary is stale, forcing a network refresh");
            return SyncingNetworkState::enter(shared_state, SyncMode::Mandatory).await;
        }

        let vpn_api_client = shared_state.vpn_api_client.clone();

        let (result_tx, result_rx) = oneshot::channel();
        let sync_cancel_token = CancellationToken::new();

        let _syncing_state_handle =
            tokio::spawn(sync_cancel_token.child_token().run_until_cancelled_owned(
                SyncingLocalState::check_account(
                    result_tx,
                    vpn_api_client,
                    vpn_api_account,
                    device,
                    summary,
                ),
            ));

        (
            Box::new(Self {
                result_rx: result_rx.fuse(),
                sync_cancel_token: Some(sync_cancel_token.drop_guard()),
            }),
            PrivateAccountControllerState::Syncing,
        )
    }

    async fn check_account(
        result_tx: oneshot::Sender<Result<bool, SyncError>>,
        vpn_api_client: VpnApiClient,
        vpn_api_account: Arc<VpnAccount>,
        device: Device,
        summary: VpnAccountSummary,
    ) {
        // Checking that the account is active
        let result = if !summary.is_account_active() {
            Err(SyncError::InactiveAccount(
                summary.account_status.to_string(),
            ))
        } else if summary.is_subscription_pending() {
            // subscription exists but is not yet active (e.g. cash payment still processing)
            Err(SyncError::PendingSubscription)
        } else if !summary.is_subscription_active() {
            // that there is an active subscription
            Err(SyncError::InactiveSubscription)
        } else if !summary.is_device_active {
            // that the device is registered or there is a spot left for it with fair usage
            if summary.remaining_devices == 0 {
                Err(SyncError::MaxDeviceReached) // Early detection of max device reached
            } else if !summary.fair_usage_left() {
                Err(SyncError::FairUsageDepleted)
            } else {
                Self::register_device(&vpn_api_client, &vpn_api_account, &device).await
            }
        } else if !summary.time_synced {
            Err(SyncError::DeviceTimeDesynced)
        } else {
            Ok(false)
        };

        result_tx.send(result).ok();
    }

    async fn register_device(
        vpn_api_client: &VpnApiClient,
        vpn_api_account: &VpnAccount,
        device: &Device,
    ) -> Result<bool, SyncError> {
        vpn_api_client
            .register_device(vpn_api_account, device)
            .await
            .map_err(|err| SyncError::UnregisteredDevice {
                details: err.to_string(),
            })?;
        Ok(true) // We just registered the device, we must update the summary (no need for a full refetch)
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for SyncingLocalState {
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

            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextAccountControllerState::NewState(OfflineState::enter())
                } else {
                    NextAccountControllerState::SameState(self)
                }
            }

            sync_result = &mut self.result_rx => {
                match sync_result {
                    Ok(Ok(device_registration)) => {
                        // If we just registered the device, reflect that in the cached summary
                        // (memory + disk) so a restart doesn't try to register again, without
                        // needing a full re-fetch.
                        if device_registration
                            && let Some(mut summary) = shared_state.vpn_account_summary.clone()
                            && !summary.is_device_active
                        {
                            summary.is_device_active = true;
                            // Guarded above: we only register when a slot is free, so this is > 0.
                            summary.remaining_devices = summary.remaining_devices.saturating_sub(1);
                            shared_state.store_summary(summary);
                        }
                        NextAccountControllerState::NewState(ReadyState::enter(shared_state).await)
                    }
                    Ok(Err(err)) => {
                        // The account doesn't check out
                        match err {
                            SyncError::PendingSubscription => NextAccountControllerState::NewState(PendingSubscriptionState::enter()),
                            err => {
                                NextAccountControllerState::NewState(ErrorState::enter(shared_state, err.into_error_reason()).await)
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("No result from local sync, task probably got cancelled : {e}");
                        NextAccountControllerState::SameState(self)
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
                            NextAccountControllerState::NewState(LoggedOutState::enter(shared_state).await)
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
                    AccountCommand::ObtainTicketbooks(return_sender) => return_sender.send(Err(AccountCommandError::AccountNotDecentralised)),
                    AccountCommand::RefreshAccountState(return_sender, force) => {
                        return_sender.send(Ok(()));
                        return if shared_state.firewall_active {
                            NextAccountControllerState::SameState(self)
                        } else {
                            if force {
                                shared_state.mark_summary_as_stale();
                                return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Mandatory).await);
                            } else {
                                return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic).await);
                            }
                        }
                    },
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                        return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Mandatory).await);
                    },

                    AccountCommand::VpnApiFirewallDown(return_sender) =>  {
                        return_sender.send(Ok(()));
                        // No-op if the firewall was already down
                        if shared_state.set_firewall_state(false) {
                            return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic).await);
                        }
                    },

                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        if shared_state.set_firewall_state(true) {
                            // Explicitly cancel the check task since the same state persists; the only
                            // network interaction (device registration) must not run while firewalled.
                            self.sync_cancel_token.take();
                        }
                        return_sender.send(Ok(()));
                    },

                    AccountCommand::Common(common_command) => {
                        common_handler::handle_common_command(common_command, shared_state).await
                    },
                }
                NextAccountControllerState::SameState(self)
            }
        }
    }
}

#[derive(Debug, strum::Display)]
enum SyncError {
    InactiveAccount(String),
    UnregisteredDevice { details: String },
    InactiveSubscription,
    PendingSubscription,
    DeviceTimeDesynced,
    MaxDeviceReached,
    FairUsageDepleted,
}

impl SyncError {
    /// Returns the corresponding error reason for the error state
    fn into_error_reason(self) -> AccountControllerErrorStateReason {
        use SyncError::*;
        match self {
            PendingSubscription => AccountControllerErrorStateReason::AccountStatusNotActive {
                status: "pending".into(),
            },
            InactiveAccount(status) => {
                AccountControllerErrorStateReason::AccountStatusNotActive { status }
            }
            InactiveSubscription => AccountControllerErrorStateReason::InactiveSubscription,
            DeviceTimeDesynced => AccountControllerErrorStateReason::DeviceTimeDesynced,
            MaxDeviceReached => AccountControllerErrorStateReason::MaxDeviceReached,
            FairUsageDepleted => AccountControllerErrorStateReason::BandwidthExceeded {
                context: SYNCING_LOCAL_STATE_CONTEXT.into(),
            },
            UnregisteredDevice { details } => AccountControllerErrorStateReason::ApiFailure {
                context: SYNCING_LOCAL_STATE_CONTEXT.into(),
                details: format!("Error registering device : {details}"),
            },
        }
    }
}
