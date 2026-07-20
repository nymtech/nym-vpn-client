// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(super) mod local_state;
pub(super) mod network_state;

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
