// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    commands::{AccountCommand, UpgradeModeCommand, common_handler, handler},
    shared_state::SharedAccountState,
    state_machine::{
        AccountControllerStateHandler, ErrorState, LoggedOutState, NextAccountControllerState,
        PrivateAccountControllerState, RequestingZkNymsState, SyncMode, SyncingNetworkState,
        UPGRADE_MODE_DEFAULT_REFRESH_INTERVAL,
    },
};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_lib_types::{AccountCommandError, AccountControllerErrorStateReason};
use std::pin::Pin;
use time::OffsetDateTime;
use tokio::{sync::mpsc, time::Sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

const UM_STATE_CONTEXT: &str = "UPGRADE_MODE_STATE";

/// UpgradeModeState
/// We end up in this state if the system is undergoing an upgrade mode,
/// when it is impossible to obtain a zk-nym for claiming bandwidth.
///
/// Possible next state:
/// - RequestingZkNymsState : We go into that state on a timer,
///   to make sure the above still holds and so that we could refresh our JWT if it's close to expiry.
/// - ErrorState : An error happened, preventing us to proceed.
/// - LoggedOutState : A successful forget account command was handled
/// - SyncingNetworkState : A successful disable upgrade mode command was handled
pub struct UpgradeModeState {
    refresh_timer: Pin<Box<Sleep>>,
}

impl UpgradeModeState {
    pub async fn enter<C: ConnectivityMonitor>(
        shared_state: &SharedAccountState<C>,
    ) -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        // determine the refresh interval based on the remaining validity of the retrieved credential
        let refresh_interval = match shared_state
            .credential_storage
            .try_retrieve_upgrade_mode_jwt()
            .await
        {
            Err(err) => {
                return ErrorState::enter(AccountControllerErrorStateReason::storage_err(
                    UM_STATE_CONTEXT,
                    err,
                ));
            }
            // we should only ever enter UpgradeModeState after we have just retrieved and saved
            // an upgrade mode credential, so this branch **theoretically** should be unreachable
            Ok(None) => {
                return ErrorState::enter(AccountControllerErrorStateReason::Internal {
                    context: "UpgradeModeState".to_string(),
                    details: "entered 'UpgradeModeState' with no upgrade mode credentials"
                        .to_string(),
                });
            }
            Ok(Some((_, Some(expiration)))) => {
                let until_expiration = expiration - OffsetDateTime::now_utc();
                // we have to ensure we refresh our JWT before it expires,
                // otherwise gateways will reject it
                if until_expiration < UPGRADE_MODE_DEFAULT_REFRESH_INTERVAL {
                    // this shouldn't be possible under any circumstances...
                    // (the sql query explicitly asks for entries that are NOT expired)
                    // so when it eventually DOES happen, make sure we attempt to refresh the state immediately
                    // instead of crashing
                    if until_expiration.is_negative() {
                        info!("the retrieved upgrade mode JWT expired immediately");
                        return SyncingNetworkState::enter(shared_state, SyncMode::Optimistic);
                    }
                    until_expiration.unsigned_abs()
                } else {
                    UPGRADE_MODE_DEFAULT_REFRESH_INTERVAL
                }
            }
            Ok(Some((_, None))) => UPGRADE_MODE_DEFAULT_REFRESH_INTERVAL,
        };

        let refresh_timer = Box::pin(tokio::time::sleep(refresh_interval));

        (
            Box::new(UpgradeModeState { refresh_timer }),
            PrivateAccountControllerState::UpgradeMode,
        )
    }

    // I don't like that return type either
    async fn on_exit<C: ConnectivityMonitor>(
        shared_state: &mut SharedAccountState<C>,
    ) -> Result<
        (),
        (
            Box<dyn AccountControllerStateHandler<C>>,
            PrivateAccountControllerState,
        ),
    > {
        // we no longer need our upgrade mode credentials - purge them
        if let Err(err) = shared_state
            .credential_storage
            .remove_upgrade_mode_jwts()
            .await
        {
            return Err(ErrorState::enter(
                AccountControllerErrorStateReason::storage_err(UM_STATE_CONTEXT, err),
            ));
        }
        Ok(())
    }

    async fn handle_account_command<C: ConnectivityMonitor>(
        self: Box<Self>,
        command: AccountCommand,
        shared_state: &mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C> {
        match command {
            AccountCommand::CreateAccount(return_sender) => {
                return_sender.send(Err(AccountCommandError::ExistingAccount))
            }
            AccountCommand::StoreAccount(return_sender, _) => {
                return_sender.send(Err(AccountCommandError::ExistingAccount))
            }
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
                };
            }
            AccountCommand::UnregisterDevice(return_sender) => {
                return_sender.send(handler::handle_try_unregister_device(shared_state).await);
            }
            AccountCommand::WipeLocalAccountData(return_sender) => {
                let res = handler::handle_wipe_local_account_data(shared_state).await;
                let error = res.is_err();
                return_sender.send(res);
                return if error {
                    NextAccountControllerState::SameState(self)
                } else {
                    NextAccountControllerState::NewState(LoggedOutState::enter())
                };
            }
            AccountCommand::LinkAccount(return_sender, privy_account) => {
                let res = handler::handle_link_account(shared_state, privy_account).await;
                return_sender.send(res);
            }
            AccountCommand::RotateKeys(return_sender) => {
                let res = handler::handle_rotate_keys(shared_state).await;
                return_sender.send(res);
            }
            AccountCommand::AccountBalance(return_sender) => {
                return_sender.send(Err(AccountCommandError::AccountNotDecentralised))
            }
            AccountCommand::ObtainTicketbooks(return_sender, _) => {
                return_sender.send(Err(AccountCommandError::AccountNotDecentralised))
            }
            AccountCommand::ResetDeviceIdentity(return_sender, seed) => {
                let res = handler::handle_reset_device_identity(shared_state, seed).await;
                let error = res.is_err();
                return_sender.send(res);
                return if error {
                    NextAccountControllerState::SameState(self)
                } else {
                    if let Err(error_state) = Self::on_exit(shared_state).await {
                        return error_state.into();
                    }
                    return NextAccountControllerState::NewState(SyncingNetworkState::enter(
                        shared_state,
                        SyncMode::Mandatory,
                    ));
                };
            }
            AccountCommand::RefreshAccountState(return_sender, force) => {
                return_sender.send(Ok(()));
                if shared_state.firewall_active {
                    warn!("firewall is active - unable to refresh account state");
                    return NextAccountControllerState::SameState(self);
                } else {
                    if let Err(error_state) = Self::on_exit(shared_state).await {
                        return error_state.into();
                    }
                    if force {
                        shared_state.mark_summary_as_stale();
                        return NextAccountControllerState::NewState(SyncingNetworkState::enter(
                            shared_state,
                            SyncMode::Mandatory,
                        ));
                    } else {
                        return NextAccountControllerState::NewState(SyncingNetworkState::enter(
                            shared_state,
                            SyncMode::Optimistic,
                        ));
                    }
                }
            }
            AccountCommand::VpnApiFirewallDown(return_sender) => {
                shared_state.firewall_active = false;
                return_sender.send(Ok(()));
            }
            AccountCommand::VpnApiFirewallUp(return_sender) => {
                shared_state.firewall_active = true;
                return_sender.send(Ok(()));
            }
            AccountCommand::Common(common_command) => {
                common_handler::handle_common_command(common_command, shared_state).await
            }
            AccountCommand::UpgradeMode(upgrade_mode_command) => match upgrade_mode_command {
                UpgradeModeCommand::GetUpgradeModeEnabled(return_sender) => {
                    return_sender.send(Ok(true))
                }
                UpgradeModeCommand::DisableUpgradeMode(return_sender) => {
                    return_sender.send(Ok(()));
                    if let Err(error_state) = Self::on_exit(shared_state).await {
                        return error_state.into();
                    }
                    return NextAccountControllerState::NewState(SyncingNetworkState::enter(
                        shared_state,
                        SyncMode::Optimistic,
                    ));
                }
            },
        }

        NextAccountControllerState::SameState(self)
    }
}

#[async_trait::async_trait]
impl<C: ConnectivityMonitor> AccountControllerStateHandler<C> for UpgradeModeState {
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
            _ = &mut self.refresh_timer => {
                debug!("attempting to retrieve a zk-nym/refresh upgrade mode attestation");
                // note: we wouldn't have entered `UpgradeModeState` if we didn't have any fair usage left
                return NextAccountControllerState::NewState(RequestingZkNymsState::enter(shared_state, 0, true))
            }
            Some(command) = command_rx.recv() => {
                self.handle_account_command(command, shared_state).await
            }
        }
    }
}
