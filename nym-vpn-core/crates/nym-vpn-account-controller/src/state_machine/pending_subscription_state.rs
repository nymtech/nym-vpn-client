// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::pin::Pin;

use crate::{
    SharedAccountState,
    commands::{AccountCommand, common_handler, handler},
    state_machine::{
        ACCOUNT_UPDATE_INTERVAL_ERROR, AccountControllerStateHandler, LoggedOutState,
        NextAccountControllerState, OfflineState, PrivateAccountControllerState, SyncMode,
        SyncingNetworkState,
    },
};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_lib_types::AccountCommandError;
use tokio::{sync::mpsc, time::Sleep};
use tokio_util::sync::CancellationToken;

/// PendingSubscriptionState
/// The account has a subscription that is pending (e.g. a cash payment still processing).
/// We wait and then re-enter the syncing state to check if the subscription has become active.
///
/// Possible next state:
/// - SyncingNetworkState: on timer expiry or a RefreshAccountState command (optimistic re-sync);
///   a ForceRefreshAccountState command drops the cache and re-syncs in mandatory mode
/// - OfflineState: connectivity monitor reports offline
/// - LoggedOutState: successfully handled a forget_account command
pub struct PendingSubscriptionState {
    refresh_timer: Pin<Box<Sleep>>,
}

impl PendingSubscriptionState {
    pub fn enter<C: ConnectivityMonitor>() -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        let refresh_timer = Box::pin(tokio::time::sleep(ACCOUNT_UPDATE_INTERVAL_ERROR));
        tracing::debug!("Account Controller entering pending subscription state");
        (
            Box::new(Self { refresh_timer }),
            PrivateAccountControllerState::PendingSubscription,
        )
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for PendingSubscriptionState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C> {
        tokio::select! {
            _ = &mut self.refresh_timer => {
                if shared_state.firewall_active {
                    tracing::debug!("VPN API is firewalled, timed account syncing skipped");
                    return NextAccountControllerState::NewState(PendingSubscriptionState::enter());
                } else {
                    return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic).await);
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
                            return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic).await);
                        } else {
                            return NextAccountControllerState::NewState(LoggedOutState::enter(shared_state).await);
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
                    AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                        let res = handler::handle_reset_device_identity(shared_state, seed).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        if error {
                            return NextAccountControllerState::SameState(self);
                        } else {
                            return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Mandatory).await);
                        }
                    },
                    AccountCommand::RefreshAccountState(return_sender, force) => {
                        return_sender.send(Ok(()));
                        if shared_state.firewall_active {
                            return NextAccountControllerState::SameState(self);
                        } else {
                            if force {
                                shared_state.mark_summary_as_stale();
                                return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Mandatory).await);
                            } else {
                                return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic).await);
                            }
                        }
                    },
                    AccountCommand::VpnApiFirewallDown(return_sender) => {
                        shared_state.set_firewall_state(false);
                        return_sender.send(Ok(()));
                    },
                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        shared_state.set_firewall_state(true);
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
