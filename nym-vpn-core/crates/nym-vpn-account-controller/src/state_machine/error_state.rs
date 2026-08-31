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
use nym_vpn_lib_types::{AccountCommandError, AccountControllerErrorStateReason};
use tokio::{sync::mpsc, time::Sleep};
use tokio_util::sync::CancellationToken;

/// ErrorState
/// We encountered something that doesn't allow us to make any progress.
/// This can range from internal issue, storage failure, API failure or unregistered account, expired subsciprions etc.
/// The full list of reason is available in the AccountControllerErrorStateReason enum
///
/// Crucially, we are online and an account is stored.
///
/// Possible next state :
/// - SyncingNetworkState : On a timer or a refresh command we optimistically re-sync to see if the
///   problem persists; a force refresh drops the cached summary and re-syncs in mandatory mode
/// - OfflineState : the connectivity monitor is telling we're not connected
/// - LoggedOutState : We successfully handled a forget_account command
pub struct ErrorState {
    refresh_timer: Pin<Box<Sleep>>,
    reason: AccountControllerErrorStateReason,
}

impl ErrorState {
    pub async fn enter<C: ConnectivityMonitor>(
        shared_state: &mut SharedAccountState<C>,
        reason: AccountControllerErrorStateReason,
    ) -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        // We can't make progress in the error state, so the bandwidth controller shouldn't keep a
        // fetcher around trying to top up.
        let _ = shared_state.clear_credential_fetcher().await;

        let refresh_timer = Box::pin(tokio::time::sleep(ACCOUNT_UPDATE_INTERVAL_ERROR));
        tracing::error!("Account Controller entering error state : {reason:#?}");
        (
            Box::new(Self {
                refresh_timer,
                reason: reason.clone(),
            }),
            PrivateAccountControllerState::Error(reason),
        )
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for ErrorState {
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
                    return NextAccountControllerState::NewState(ErrorState::enter(shared_state, self.reason).await);
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

                    AccountCommand::VpnApiFirewallDown(return_sender) =>  {
                        return_sender.send(Ok(()));
                        if shared_state.set_firewall_state(false) {
                            return NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic).await);
                        }
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
