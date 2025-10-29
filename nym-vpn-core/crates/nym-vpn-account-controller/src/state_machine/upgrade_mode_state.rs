// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    commands::AccountCommand,
    shared_state::SharedAccountState,
    state_machine::{
        AccountControllerStateHandler, NextAccountControllerState, PrivateAccountControllerState,
    },
};
use nym_offline_monitor::ConnectivityMonitor;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// UpgradeModeState
/// We end up in this state if the system is undergoing an upgrade mode,
/// when it is impossible to obtain a zk-nym for claiming bandwidth.
///
/// Possible next state:
/// - ErrorState : An error happened, preventing us to proceed.
/// - LoggedOutState : A successful forget account command was handled
pub struct UpgradeModeState;

impl UpgradeModeState {
    pub fn enter<C: ConnectivityMonitor>() -> (
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    ) {
        todo!("check and refresh storage");

        (
            Box::new(UpgradeModeState),
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
