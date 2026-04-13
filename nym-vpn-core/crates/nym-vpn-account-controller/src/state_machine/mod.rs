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
mod upgrade_mode_state;
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

/// Account stored, online, determining if we can't connect or not
pub(crate) use syncing_state::SyncingState;

/// We're in the process of attempting to acquire a zk-nym
pub(crate) use syncing_state::requesting_zknym_state::RequestingZkNymsState;

/// Account is operating independently of VPN API
pub(crate) use decentralised_state::DecentralisedState;

/// The system is undergoing an upgrade mode, where zk-nyms can't be issued
pub(crate) use upgrade_mode_state::UpgradeModeState;

// The interval at which we update the account state
const ACCOUNT_UPDATE_INTERVAL: Duration = Duration::from_secs(2 * 60);

// The interval at which we attempt to exit the upgrade mode by trying to get a new zk-nym instead
// (note: this does not prevent bandwidth controller from notifying us directly about the UM being over)
const UPGRADE_MODE_DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_secs(10 * 60);

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
            PrivateAccountControllerState::UpgradeMode => Self::UpgradeMode,
            PrivateAccountControllerState::PendingSubscription => Self::PendingSubscription,
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
    UpgradeMode,
    PendingSubscription,
    Error(AccountControllerErrorStateReason),
    RequestingZkNyms,
}
