// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_lib_types::{AccountControllerErrorStateReason, AccountControllerState};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{SharedAccountState, commands::AccountCommand};

mod decentralised_state;
mod error_state;
mod logged_out_state;
mod offline_state;
mod ready_state;
mod syncing_state;

// Account Controller state machine available states

/// Account stored, online, can't proceed without user action and/or temporary failure somewhere
pub(crate) use error_state::ErrorState;

/// No account stored, online
pub use logged_out_state::LoggedOutState;

/// Maybe account stored, offline,
pub use offline_state::OfflineState;

/// Account stored, online, ready to connect
pub(crate) use ready_state::ReadyState;

/// Account stored, online, determining if we can't connect or not
pub(crate) use syncing_state::SyncingState;

// The interval at which we update the account state
const ACCOUNT_UPDATE_INTERVAL: Duration = Duration::from_secs(2 * 60);

#[async_trait::async_trait]
pub(crate) trait AccountControllerStateHandler<C: ConnectivityMonitor>: Send {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<AccountCommand>,
        shared_state: &'async_trait mut SharedAccountState<C>,
    ) -> NextAccountControllerState<C>;
}

pub(crate) enum NextAccountControllerState<C: ConnectivityMonitor> {
    NewState(
        (
            Box<dyn AccountControllerStateHandler<C>>,
            PrivateAccountControllerState,
        ),
    ),
    SameState(Box<dyn AccountControllerStateHandler<C>>),
    Finished,
}

impl From<PrivateAccountControllerState> for AccountControllerState {
    fn from(value: PrivateAccountControllerState) -> Self {
        match value {
            PrivateAccountControllerState::Offline => Self::Offline,
            PrivateAccountControllerState::Syncing => Self::Syncing,
            PrivateAccountControllerState::LoggedOut => Self::LoggedOut,
            PrivateAccountControllerState::ReadyToConnect => Self::ReadyToConnect,
            PrivateAccountControllerState::Decentralised => Self::Decentralised,
            PrivateAccountControllerState::Error(reason) => Self::Error(reason),
            PrivateAccountControllerState::RequestingZkNyms => Self::RequestingZkNyms,
        }
    }
}

/// Private enum describing the account controller state
#[derive(Debug, Clone)]
pub(super) enum PrivateAccountControllerState {
    Offline,
    Syncing,
    LoggedOut,
    ReadyToConnect,
    Decentralised,
    Error(AccountControllerErrorStateReason),
    RequestingZkNyms,
}
