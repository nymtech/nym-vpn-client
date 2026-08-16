// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    SharedAccountState,
    commands::{
        AccountCommand, CommonCommand, ReturnSender, UpgradeModeCommand, common_handler, handler,
    },
    state_machine::{
        AccountControllerStateHandler, LoggedOutState, NextAccountControllerState,
        PrivateAccountControllerState, SyncMode, SyncingNetworkState,
    },
};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_lib_types::AccountCommandError;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::warn;

/// OfflineState
///
/// The connectivity monitor told us it had no connectivity.
/// There is no assumption wrt to stored account and registered device
/// Most of the commands can't be handled because of the lack of connectivity
///
/// When the connectivity is restored, we go into SyncingNetworkState to get back on our feet
///
/// Possible next state :
/// - SyncingNetworkState : Connectivity is back
///
pub struct OfflineState;

impl OfflineState {
    pub fn enter<C: ConnectivityMonitor>() -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        (Box::new(Self), PrivateAccountControllerState::Offline)
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for OfflineState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C> {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                // Intentionnally no command grouping for clarity
                match command {
                    AccountCommand::CreateAccount(return_sender) => {
                        return_sender.send(handler::handle_create_account(shared_state).await)
                    },
                    AccountCommand::StoreAccount(return_sender, account) => {
                        return_sender.send(handler::handle_store_account(shared_state, account).await)
                    },
                    AccountCommand::RegisterAccount(return_sender, _, _) => return_no_connectivity(return_sender),
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
                    AccountCommand::UnregisterDevice(return_sender) => {
                        // Best-effort: we're offline, can't reach the API anyway
                        return_sender.send(Ok(()));
                    },
                    AccountCommand::WipeLocalAccountData(return_sender) => {
                        let res = handler::handle_wipe_local_account_data(shared_state).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        return if error {
                            NextAccountControllerState::SameState(self)
                        } else {
                            NextAccountControllerState::NewState(LoggedOutState::enter())
                        }
                    },
                    AccountCommand::LinkAccount(return_sender, _) => return_no_connectivity(return_sender),
                    AccountCommand::RotateKeys(return_sender) => {
                        return_sender.send(handler::handle_rotate_keys(shared_state).await)
                    },
                    AccountCommand::AccountBalance(return_sender) => return_no_connectivity(return_sender),
                    AccountCommand::ObtainTicketbooks(return_sender, _) => return_no_connectivity(return_sender),
                    // Same comment as above
                    AccountCommand::ResetDeviceIdentity(return_sender, _) => return_no_connectivity(return_sender),

                    AccountCommand::RefreshAccountState(return_sender, _) => return_no_connectivity(return_sender),

                    AccountCommand::VpnApiFirewallDown(return_sender) =>  {
                        shared_state.firewall_active = false;
                        return_sender.send(Ok(()));
                    },
                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        shared_state.firewall_active = true;
                        return_sender.send(Ok(()));
                    },


                    AccountCommand::Common(common_command) => {
                        match common_command {
                            CommonCommand::GetStoredAccount(return_sender) => return_sender.send(common_handler::handle_get_stored_account(shared_state).await),
                            CommonCommand::GetDeviceIdentity(return_sender) => return_sender.send(common_handler::handle_get_device_identity(shared_state)),
                            CommonCommand::GetAccountIdentity(return_sender) => return_sender.send(common_handler::handle_get_account_identity(shared_state)),
                            CommonCommand::GetCanonicalAccountIdentity(return_sender) => return_sender.send(common_handler::handle_get_canonical_account_identity(shared_state).await),
                            CommonCommand::GetAccountMode(return_sender) => return_sender.send(common_handler::handle_get_account_mode(shared_state)),
                            CommonCommand::GetUsage(return_sender) => return_no_connectivity(return_sender),
                            CommonCommand::GetDevices(return_sender) => return_no_connectivity(return_sender),
                            CommonCommand::GetActiveDevices(return_sender) => return_no_connectivity(return_sender),
                            CommonCommand::GetAvailableTickets(return_sender) => return_no_connectivity(return_sender),
                            CommonCommand::GetAccountSummary(return_sender) => return_sender.send(common_handler::handle_get_account_summary(shared_state).await),
                            CommonCommand::GetDeeplink(return_sender, params) => return_sender.send(common_handler::handle_get_deeplink(shared_state, params).await),
                            CommonCommand::GetAutologinDeeplink(return_sender, params) => return_sender.send(common_handler::handle_get_autologin_deeplink(shared_state, params).await),
                            CommonCommand::DeriveDeeplinkMnemonic(return_sender, deeplink_callback_url) => return_sender.send(common_handler::handle_derive_deeplink_mnemonic(shared_state, deeplink_callback_url).await),
                        }

                    },
                    AccountCommand::UpgradeMode(upgrade_mode_command) => match upgrade_mode_command {
                        UpgradeModeCommand::GetUpgradeModeEnabled(return_sender) => {
                            return_sender.send(Ok(false))
                        }
                        UpgradeModeCommand::DisableUpgradeMode(return_sender) => {
                            warn!(
                                "received unexpected command to disable upgrade mode while in 'OfflineState' state"
                            );
                            return_sender.send(Ok(()))
                        }
                    },

                }
                NextAccountControllerState::SameState(self)
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextAccountControllerState::SameState(self)
                } else {
                    NextAccountControllerState::NewState(SyncingNetworkState::enter(shared_state, SyncMode::Optimistic))
                }
            }
            _ = shutdown_token.cancelled() => {
                NextAccountControllerState::Finished
            }
        }
    }
}

fn return_no_connectivity<S>(result_tx: ReturnSender<S, AccountCommandError>)
where
    S: std::fmt::Debug + std::marker::Send,
{
    result_tx.send(Err(AccountCommandError::Offline))
}
