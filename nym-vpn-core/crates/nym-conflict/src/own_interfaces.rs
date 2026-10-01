// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Tracks the OS interface names NymVPN itself currently owns, so
//! [`crate::ConflictCheck::CompetingVpn`] never reports NymVPN's own tunnel
//! as a competing VPN. Populated by whoever brings the interface up (e.g.
//! `nym-vpn-lib`'s `RouteHandler`), and handed to [`crate::detect`], which
//! consults it before counting an interface as a competitor.

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Default)]
pub struct OwnInterfaces {
    names: Arc<Mutex<HashSet<String>>>,
}

impl OwnInterfaces {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that `name` is one of NymVPN's own tunnel interfaces.
    pub fn mark(&self, name: impl Into<String>) {
        self.lock().insert(name.into());
    }

    /// Reverse of [`Self::mark`], to be called once the interface is torn down.
    pub fn unmark(&self, name: &str) {
        self.lock().remove(name);
    }

    /// Whether `name` is currently one of NymVPN's own tunnel interfaces.
    pub fn contains(&self, name: &str) -> bool {
        self.lock().contains(name)
    }

    /// Forget every interface recorded via [`Self::mark`]. Called once NymVPN
    /// has torn down its own routes, since at most one set of own interfaces
    /// is ever active at a time.
    pub fn clear(&self) {
        self.lock().clear();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashSet<String>> {
        self.names
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_and_unmark() {
        let own = OwnInterfaces::new();
        assert!(!own.contains("utun-test-a"));

        own.mark("utun-test-a");
        own.mark("utun-test-b");
        assert!(own.contains("utun-test-a"));
        assert!(own.contains("utun-test-b"));

        own.unmark("utun-test-a");
        assert!(!own.contains("utun-test-a"));
        assert!(own.contains("utun-test-b"));
    }

    #[test]
    fn clear_forgets_everything() {
        let own = OwnInterfaces::new();
        own.mark("utun-test-a");
        own.clear();
        assert!(!own.contains("utun-test-a"));
    }

    #[test]
    fn clones_share_the_same_set() {
        let own = OwnInterfaces::new();
        let clone = own.clone();

        own.mark("utun-test-a");
        assert!(clone.contains("utun-test-a"));

        clone.unmark("utun-test-a");
        assert!(!own.contains("utun-test-a"));
    }
}
