// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use crate::{
    SharedAccountState,
    commands::{AccountCommand, UpgradeModeCommand, common_handler, handler},
    state_machine::{
        AccountControllerStateHandler, DecentralisedState, ErrorState, LoggedOutState,
        NextAccountControllerState, OfflineState, PendingSubscriptionState,
        PrivateAccountControllerState, RequestingZkNymsState,
        syncing_state::{SyncMode, SyncingNetworkState},
    },
};
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
use tracing::warn;

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
/// Failed checks go straight to the error state (or `PendingSubscriptionState` for a pending
/// subscription). Whether the cache is trustworthy is decided only at entry via [`VpnAccountSummary::is_stale`].
///
/// Possible next state :
/// - LoggedOutState : No account is stored
/// - SyncingNetworkState : No summary is cached, or the cache is stale and must be refreshed
/// - RequestingZkNymsState : Everything checks out, we just need to top up our zk-nyms
/// - PendingSubscriptionState : The subscription exists but is not yet active
/// - ErrorState : One of the checks failed in a non-recoverable way
/// - OfflineState : the connectivity monitor is telling we're not connected
/// - DecentralisedState : The loaded account is set to "decentralised" mode
pub(crate) struct SyncingLocalState {
    result_rx: oneshot::Receiver<LocalSyncCheckResult>,
    sync_cancel_token: Option<DropGuard>,
}

impl SyncingLocalState {
    pub(crate) fn enter<C: ConnectivityMonitor>(
        shared_state: &SharedAccountState<C>,
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
                context: SYNCING_LOCAL_STATE_CONTEXT.into(),
                details: "Logged in, but no device keys".into(),
            });
        };

        // Nothing to check locally without a cached summary: go fetch one from the network.
        let Some(summary) = shared_state.vpn_account_summary.clone() else {
            return SyncingNetworkState::enter(shared_state, SyncMode::Mandatory);
        };

        // Can we work with this summary? No => mandatory sync.
        if summary.is_stale(SUMMARY_STALE_AFTER) {
            tracing::debug!("Cached account summary is stale, forcing a network refresh");
            return SyncingNetworkState::enter(shared_state, SyncMode::Mandatory);
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
                result_rx,
                sync_cancel_token: Some(sync_cancel_token.drop_guard()),
            }),
            PrivateAccountControllerState::Syncing,
        )
    }

    async fn check_account(
        result_tx: oneshot::Sender<LocalSyncCheckResult>,
        vpn_api_client: VpnApiClient,
        vpn_api_account: Arc<VpnAccount>,
        device: Device,
        summary: VpnAccountSummary,
    ) {
        let result = match local_sync_outcome(&summary) {
            LocalSyncOutcome::Ready => LocalSyncCheckResult::ContinueToZkNyms {
                device_registration: false,
            },
            LocalSyncOutcome::RegisterDevice => match Self::register_device(
                &vpn_api_client,
                &vpn_api_account,
                &device,
            )
            .await
            {
                Ok(()) => LocalSyncCheckResult::ContinueToZkNyms {
                    device_registration: true,
                },
                Err(reason) => LocalSyncCheckResult::Failed(reason),
            },
            LocalSyncOutcome::PendingSubscription => LocalSyncCheckResult::PendingSubscription,
            LocalSyncOutcome::Failed(reason) => LocalSyncCheckResult::Failed(reason),
        };

        result_tx.send(result).ok();
    }

    async fn register_device(
        vpn_api_client: &VpnApiClient,
        vpn_api_account: &VpnAccount,
        device: &Device,
    ) -> Result<(), AccountControllerErrorStateReason> {
        vpn_api_client
            .register_device(vpn_api_account, device)
            .await
            .map(|_| ())
            .map_err(|err| AccountControllerErrorStateReason::ApiFailure {
                context: SYNCING_LOCAL_STATE_CONTEXT.into(),
                details: format!("Error registering device : {err}"),
            })
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
                    Ok(LocalSyncCheckResult::ContinueToZkNyms { device_registration }) => {
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
                        NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state, 0, false))
                    }
                    Ok(LocalSyncCheckResult::PendingSubscription) => {
                        NextAccountControllerState::NewState(PendingSubscriptionState::enter())
                    }
                    Ok(LocalSyncCheckResult::Failed(reason)) => {
                        NextAccountControllerState::NewState(ErrorState::enter(reason))
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
                    AccountCommand::RefreshAccountState(return_sender, force) => {
                        return_sender.send(Ok(()));
                        return if shared_state.firewall_active {
                            NextAccountControllerState::SameState(self)
                        } else {
                            if force {
                                shared_state.mark_summary_as_stale();
                                return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Mandatory));
                            } else {
                                return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic));
                            }
                        }
                    },
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        return_sender.send(handler::handle_reset_device_identity(shared_state, seed).await);
                        return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Mandatory));
                    },

                    AccountCommand::VpnApiFirewallDown(return_sender) =>  {
                        return_sender.send(Ok(()));
                        // No-op if the firewall was already down
                        if shared_state.firewall_active {
                            shared_state.firewall_active = false;
                            return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic));
                        }
                    },

                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        shared_state.firewall_active = true;
                        // Explicitly cancel the check task since the same state persists; the only
                        // network interaction (device registration) must not run while firewalled.
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
                                "received unexpected command to disable upgrade mode while in 'SyncingLocalState' state"
                            );
                            return_sender.send(Ok(()))
                        }
                    },
                }
                NextAccountControllerState::SameState(self)
            }
        }
    }
}

fn local_sync_outcome(summary: &VpnAccountSummary) -> LocalSyncOutcome {
    if !summary.is_account_active() {
        return LocalSyncOutcome::Failed(AccountControllerErrorStateReason::AccountStatusNotActive {
            status: summary.account_status.to_string(),
        });
    }
    if summary.is_subscription_pending() {
        return LocalSyncOutcome::PendingSubscription;
    }
    if !summary.is_subscription_active() {
        return LocalSyncOutcome::Failed(AccountControllerErrorStateReason::InactiveSubscription);
    }
    if !summary.fair_usage_left() {
        return LocalSyncOutcome::Failed(AccountControllerErrorStateReason::BandwidthExceeded {
            context: SYNCING_LOCAL_STATE_CONTEXT.into(),
        });
    }
    if !summary.is_device_active {
        if summary.remaining_devices == 0 {
            return LocalSyncOutcome::Failed(AccountControllerErrorStateReason::MaxDeviceReached);
        }
        return LocalSyncOutcome::RegisterDevice;
    }
    if !summary.time_synced {
        return LocalSyncOutcome::Failed(AccountControllerErrorStateReason::DeviceTimeDesynced);
    }
    LocalSyncOutcome::Ready
}

#[derive(Debug, PartialEq, Eq)]
enum LocalSyncOutcome {
    Ready,
    RegisterDevice,
    PendingSubscription,
    Failed(AccountControllerErrorStateReason),
}

#[derive(Debug, PartialEq, Eq)]
enum LocalSyncCheckResult {
    ContinueToZkNyms { device_registration: bool },
    PendingSubscription,
    Failed(AccountControllerErrorStateReason),
}

#[cfg(test)]
mod local_sync_outcome_tests {
    use super::{LocalSyncOutcome, local_sync_outcome};
    use nym_vpn_lib_types::{
        AccountControllerErrorStateReason, NymVpnSubscription, NymVpnSubscriptionKind,
        NymVpnSubscriptionStatus, Subscription, VpnAccountStatus, VpnAccountSummary,
    };
    use time::OffsetDateTime;

    fn active_summary(
        is_device_active: bool,
        traffic_used_gb: u64,
        traffic_limit_gb: u64,
    ) -> VpnAccountSummary {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        VpnAccountSummary {
            traffic_used_gb,
            traffic_limit_gb,
            traffic_reset_time: None,
            fair_usage_data_unavailable: false,
            account_addr: "n1test".into(),
            canonical_account_addr: None,
            auth_methods: vec![],
            account_mode: None,
            subscription: Some(Subscription {
                status: NymVpnSubscriptionStatus::Active,
                subscription: NymVpnSubscription {
                    created_on_utc: "2024-01-01T00:00:00Z".into(),
                    last_updated_utc: "2024-01-01T00:00:00Z".into(),
                    id: "sub_active".into(),
                    valid_from_utc: now - 86_400,
                    valid_until_utc: now + 30 * 86_400,
                    status: "active".into(),
                    kind: NymVpnSubscriptionKind::OneMonth,
                    is_recurring: false,
                },
            }),
            is_subscription_stacked: false,
            account_status: VpnAccountStatus::Active,
            remaining_devices: 5,
            is_device_active,
            time_synced: true,
            stale: false,
            last_synced_utc: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn local_sync_outcome_reports_depletion_for_active_device() {
        let summary = active_summary(true, 2000, 2000);
        assert_eq!(
            local_sync_outcome(&summary),
            LocalSyncOutcome::Failed(AccountControllerErrorStateReason::BandwidthExceeded {
                context: super::SYNCING_LOCAL_STATE_CONTEXT.into(),
            })
        );
    }

    #[test]
    fn local_sync_outcome_reports_depletion_before_device_registration() {
        let summary = active_summary(false, 2000, 2000);
        assert_eq!(
            local_sync_outcome(&summary),
            LocalSyncOutcome::Failed(AccountControllerErrorStateReason::BandwidthExceeded {
                context: super::SYNCING_LOCAL_STATE_CONTEXT.into(),
            })
        );
    }

    #[test]
    fn local_sync_outcome_registers_only_when_quota_remains() {
        let summary = active_summary(false, 0, 2000);
        assert_eq!(local_sync_outcome(&summary), LocalSyncOutcome::RegisterDevice);
    }

    #[test]
    fn local_sync_outcome_ready_when_checks_pass() {
        let summary = active_summary(true, 0, 2000);
        assert_eq!(local_sync_outcome(&summary), LocalSyncOutcome::Ready);
    }
}
