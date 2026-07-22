// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    commands::{
        AccountCommand, CommonCommand, ReturnSender, common_handler, decentralised_zknym_handler,
        handler,
    },
    shared_state::SharedAccountState,
    state_machine::{
        AccountControllerStateHandler, LoggedOutState, NextAccountControllerState, OfflineState,
        PrivateAccountControllerState,
    },
};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_lib_types::AccountCommandError;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;

/// DecentralisedState
/// We are operating independently of the VPN API which means:
/// - A **decentralised** account is stored
/// - The account exists on chain and has some tokens
/// - Ticketbooks are obtaining through manual deposits
///
/// Possible next state :
/// - OfflineState : the connectivity monitor is telling we're not connected
/// - LoggedOutState : We successfully handled a forget_account command
pub struct DecentralisedState;

impl DecentralisedState {
    pub fn enter<C: ConnectivityMonitor>() -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        (Box::new(Self), PrivateAccountControllerState::Decentralised)
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for DecentralisedState {
    async fn handle_event(
        self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut UnboundedReceiver<AccountCommand>,
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
            Some(command) = command_rx.recv() => {
                match command {
                    AccountCommand::ForgetAccount(return_sender) => {
                        let res = handler::handle_forget_account(shared_state).await;
                        let error = res.is_err();
                        return_sender.send(res);
                        if !error {
                            return NextAccountControllerState::NewState(LoggedOutState::enter(shared_state).await)
                        }
                    },
                    AccountCommand::RotateKeys(return_sender) => {
                        let res = handler::handle_rotate_keys(shared_state).await;
                        return_sender.send(res);
                    },
                    AccountCommand::AccountBalance(return_sender) => {
                        return_sender.send(decentralised_zknym_handler::handle_account_balance(shared_state).await);
                    }
                    AccountCommand::ObtainTicketbooks(return_sender) => {
                        return_sender.send(decentralised_zknym_handler::handle_obtain_ticketbooks(shared_state).await);
                    }
                    AccountCommand::Common(common_command) => {
                         match common_command {
                            CommonCommand::GetStoredAccount(return_sender) => return_sender.send(common_handler::handle_get_stored_account(shared_state).await),
                            CommonCommand::GetDeviceIdentity(return_sender) => return_decentralised(return_sender),
                            CommonCommand::GetAccountIdentity(return_sender) => return_sender.send(common_handler::handle_get_account_identity(shared_state)),
                            CommonCommand::GetCanonicalAccountIdentity(return_sender) => return_sender.send(common_handler::handle_get_canonical_account_identity(shared_state).await),
                            CommonCommand::GetAccountMode(return_sender) => return_sender.send(common_handler::handle_get_account_mode(shared_state)),
                            CommonCommand::GetUsage(return_sender) => return_decentralised(return_sender),
                            CommonCommand::GetDevices(return_sender) => return_decentralised(return_sender),
                            CommonCommand::GetActiveDevices(return_sender) => return_decentralised(return_sender),
                            CommonCommand::GetAccountSummary(return_sender) => return_sender.send(common_handler::handle_get_account_summary(shared_state).await),
                            CommonCommand::GetDeeplink(return_sender, params) => return_sender.send(common_handler::handle_get_deeplink(shared_state, params).await),
                            CommonCommand::GetAutologinDeeplink(return_sender, params) => return_sender.send(common_handler::handle_get_autologin_deeplink(shared_state, params).await),
                            CommonCommand::DeriveDeeplinkMnemonic(return_sender, deeplink_callback_url) => return_sender.send(common_handler::handle_derive_deeplink_mnemonic(shared_state, deeplink_callback_url).await),
                        }
                    },
                    AccountCommand::VpnApiFirewallUp(return_sender) => {
                        shared_state.set_firewall_state(true);
                        return_sender.send(Ok(()));
                    },
                    AccountCommand::VpnApiFirewallDown(return_sender) => {
                        shared_state.set_firewall_state(false);
                        return_sender.send(Ok(()));
                    },
                    other => {
                        other.return_error(AccountCommandError::AccountDecentralised);
                    }
                }
                NextAccountControllerState::SameState(self)
            }
        }
    }
}

fn return_decentralised<S>(result_tx: ReturnSender<S, AccountCommandError>)
where
    S: std::fmt::Debug + std::marker::Send,
{
    result_tx.send(Err(AccountCommandError::AccountDecentralised))
}
