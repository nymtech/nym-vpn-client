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
mod pending_subscription_state;
mod ready_state;
mod syncing_state;
//mod upgrade_mode_state;
// Account Controller state machine available states

/// Account stored, online, can't proceed without user action and/or temporary failure somewhere
pub(crate) use error_state::ErrorState;

/// Account stored, online, subscription is pending (e.g. cash payment processing)
pub(crate) use pending_subscription_state::PendingSubscriptionState;

/// No account stored, online
pub use logged_out_state::LoggedOutState;

/// Maybe account stored, offline,
pub use offline_state::OfflineState;

/// Account stored, online, ready to connect
pub(crate) use ready_state::ReadyState;

/// Account stored, online, fetching the account summary from the VPN API
pub(crate) use syncing_state::SyncingNetworkState;

/// Whether a network sync is optimistic (short timeout, cache fallback) or mandatory
pub(crate) use syncing_state::SyncMode;

/// Account is operating independently of VPN API
pub(crate) use decentralised_state::DecentralisedState;

// The interval at which we update the account state when in error state
const ACCOUNT_UPDATE_INTERVAL_ERROR: Duration = Duration::from_secs(2 * 60);

// The interval at which we update the account state when in ready state
const ACCOUNT_UPDATE_INTERVAL_READY: Duration = Duration::from_secs(60 * 60);

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

impl<C: ConnectivityMonitor>
    From<(
        Box<dyn AccountControllerStateHandler<C>>,
        PrivateAccountControllerState,
    )> for NextAccountControllerState<C>
{
    fn from(
        new_state: (
            Box<dyn AccountControllerStateHandler<C>>,
            PrivateAccountControllerState,
        ),
    ) -> Self {
        NextAccountControllerState::NewState(new_state)
    }
}

impl<C: ConnectivityMonitor> From<Box<dyn AccountControllerStateHandler<C>>>
    for NextAccountControllerState<C>
{
    fn from(state: Box<dyn AccountControllerStateHandler<C>>) -> Self {
        NextAccountControllerState::SameState(state)
    }
}

impl From<PrivateAccountControllerState> for AccountControllerState {
    fn from(value: PrivateAccountControllerState) -> Self {
        match value {
            PrivateAccountControllerState::Offline => Self::Offline,
            PrivateAccountControllerState::Syncing => Self::Syncing,
            PrivateAccountControllerState::LoggedOut => Self::LoggedOut,
            PrivateAccountControllerState::ReadyToConnect => Self::ReadyToConnect,
            PrivateAccountControllerState::Decentralised => Self::Decentralised,
            PrivateAccountControllerState::PendingSubscription => Self::PendingSubscription,
            PrivateAccountControllerState::Error(reason) => Self::Error(reason),
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
    PendingSubscription,
    Error(AccountControllerErrorStateReason),
}
