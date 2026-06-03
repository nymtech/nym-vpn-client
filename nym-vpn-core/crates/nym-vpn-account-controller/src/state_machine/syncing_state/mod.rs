// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    SharedAccountState, state_machine::NextAccountControllerState, storage::AccountStorageOp,
};
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_lib_types::VpnAccountSummary;

pub(super) mod local_state;
pub(super) mod network_state;
pub(super) mod requesting_zknym_state;

pub(crate) use network_state::SyncingNetworkState;

/// How aggressively [`SyncingNetworkState`] should try to refresh the account summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SyncMode {
    /// Best-effort refresh on a short timeout. On any failure (timeout, connection error, or an
    /// error response) we fall back to the cached summary instead of blocking or erroring.
    Optimistic,

    /// Full fetch with no cache fallback. Used for forced refreshes, stale caches, and retries,
    /// where we must obtain fresh data (or surface an error).
    Mandatory,
}

/// Build the [`VpnAccountSummary`] from the API response, then store it both in memory and on
/// disk so it survives restarts and can be re-evaluated locally.
fn store_summary<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    summary: VpnAccountSummary,
) {
    // Persist the summary alongside the mnemonic/keys (best-effort, via the storage-op
    // channel) and keep the in-memory working copy in sync.
    let _ = shared_state
        .storage_op_sender
        .send(AccountStorageOp::StoreAccountSummary(Box::new(
            summary.clone(),
        )));
    shared_state.vpn_account_summary = Some(summary);
}

fn remove_summary<C: ConnectivityMonitor>(shared_state: &mut SharedAccountState<C>) {
    // best effort
    let _ = shared_state
        .storage_op_sender
        .send(AccountStorageOp::RemoveAccountSummary);
    shared_state.vpn_account_summary = None;
}

// This way we can force an account summary cleanup, before actually entering the state
pub(crate) fn force_refresh<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> NextAccountControllerState<C> {
    // best effort
    remove_summary(shared_state);
    NextAccountControllerState::NewState(SyncingNetworkState::enter(
        shared_state,
        0,
        SyncMode::Mandatory,
    ))
}
