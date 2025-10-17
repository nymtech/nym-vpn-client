// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::commands::AccountCommand;
use crate::shared_state::SharedAccountState;
use crate::state_machine::{
    AccountControllerStateHandler, NextAccountControllerState, PrivateAccountControllerState,
};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_lib_types::UpgradeModeData;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// UpgradeModeState
/// We end up in this state if the system is undergoing an upgrade mode,
/// when it is impossible to obtain a zk-nym for claiming bandwidth.
///
/// Possible next state:
/// - ErrorState : An error happened, preventing us to proceed.
/// - LoggedOutState : A successful forget account command was handled
pub struct UpgradeModeState {
    data: Box<UpgradeModeData>,
}

impl UpgradeModeState {
    pub fn enter<C: ConnectivityMonitor>(
        upgrade_mode_data: Box<UpgradeModeData>,
    ) -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        (
            Box::new(UpgradeModeState {
                data: upgrade_mode_data,
            }),
            PrivateAccountControllerState::UpgradeMode,
        )
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
        todo!()
    }
}
