// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Tracks the OS interface names NymVPN itself currently owns, so
//! [`crate::ConflictCheck::CompetingVpn`] never reports NymVPN's own tunnel
//! as a competing VPN. Populated by whoever brings the interface up (e.g.
//! `nym-vpn-lib`'s `RouteHandler`), consulted by [`crate::vpn::detect`]
//! before counting an interface as a competitor.

use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};

fn registry() -> &'static Mutex<HashSet<String>> {
    static REGISTRY: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Record that `name` is one of NymVPN's own tunnel interfaces.
pub fn mark(name: impl Into<String>) {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(name.into());
}

/// Reverse of [`mark`], to be called once the interface is torn down.
pub fn unmark(name: &str) {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(name);
}

/// Whether `name` is currently one of NymVPN's own tunnel interfaces.
pub(crate) fn contains(name: &str) -> bool {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .contains(name)
}

/// Forget every interface recorded via [`mark`]. Called once NymVPN has
/// torn down its own routes, since at most one set of own interfaces is
/// ever active at a time.
pub fn clear() {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    // Single test function: the registry is a process-wide global, so
    // separate #[test] fns here would race against each other under the
    // default parallel test runner.
    #[test]
    fn mark_contains_unmark_and_clear() {
        assert!(!contains("utun-test-a"));

        mark("utun-test-a");
        mark("utun-test-b");
        assert!(contains("utun-test-a"));
        assert!(contains("utun-test-b"));

        unmark("utun-test-a");
        assert!(!contains("utun-test-a"));
        assert!(contains("utun-test-b"));

        clear();
        assert!(!contains("utun-test-b"));
    }
}
