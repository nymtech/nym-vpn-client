// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

/// Number of consecutive pre-handshake connection failures with the same entry
/// gateway after which the entry gateway is considered at fault.
pub const ENTRY_BLAME_STRIKE_THRESHOLD: u32 = 2;

/// Tracks consecutive connection failures that happened before the exit WireGuard
/// handshake ever completed, in order to attribute blame to the entry gateway.
///
/// A failure before the exit handshake completes is ambiguous: either hop may be at
/// fault, and each failure already blacklists the exit gateway. But when several
/// consecutive attempts fail pre-handshake through the same entry gateway while the
/// exit gateway changes, the entry gateway is the common failing link.
#[derive(Debug)]
pub struct EntryBlameTracker<T> {
    strikes: Option<(T, u32)>,
}

impl<T> Default for EntryBlameTracker<T> {
    fn default() -> Self {
        Self { strikes: None }
    }
}

impl<T: PartialEq> EntryBlameTracker<T> {
    /// Record a connection failure through the given entry gateway.
    ///
    /// `exit_handshake_completed` tells whether the exit WireGuard handshake completed
    /// at least once during the failed attempt. Returns `true` when the entry gateway
    /// has accumulated enough pre-handshake failures to be blacklisted.
    pub fn record_failure(&mut self, entry_id: T, exit_handshake_completed: bool) -> bool {
        if exit_handshake_completed {
            self.strikes = None;
            return false;
        }

        let strikes = match self.strikes.take() {
            Some((id, strikes)) if id == entry_id => strikes.saturating_add(1),
            _ => 1,
        };

        if strikes >= ENTRY_BLAME_STRIKE_THRESHOLD {
            self.strikes = None;
            true
        } else {
            self.strikes = Some((entry_id, strikes));
            false
        }
    }

    /// Clear accumulated strikes, e.g. after a successful connection.
    pub fn clear(&mut self) {
        self.strikes = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_blame_entry_on_first_prehandshake_failure() {
        let mut tracker = EntryBlameTracker::default();

        assert!(!tracker.record_failure("entry-a", false));
    }

    #[test]
    fn blames_entry_after_consecutive_prehandshake_failures_with_same_entry() {
        let mut tracker = EntryBlameTracker::default();

        assert!(!tracker.record_failure("entry-a", false));
        assert!(
            tracker.record_failure("entry-a", false),
            "second consecutive pre-handshake failure through the same entry must blame the entry"
        );
    }

    #[test]
    fn does_not_blame_entry_when_entry_changes_between_failures() {
        let mut tracker = EntryBlameTracker::default();

        assert!(!tracker.record_failure("entry-a", false));
        assert!(!tracker.record_failure("entry-b", false));
    }

    #[test]
    fn completed_exit_handshake_clears_strikes() {
        let mut tracker = EntryBlameTracker::default();

        assert!(!tracker.record_failure("entry-a", false));
        // The tunnel worked end-to-end, so the entry is not the failing link.
        assert!(!tracker.record_failure("entry-a", true));
        assert!(!tracker.record_failure("entry-a", false));
    }

    #[test]
    fn strikes_reset_after_entry_is_blamed() {
        let mut tracker = EntryBlameTracker::default();

        assert!(!tracker.record_failure("entry-a", false));
        assert!(tracker.record_failure("entry-a", false));
        assert!(
            !tracker.record_failure("entry-a", false),
            "blame must be reported once per strike streak"
        );
    }

    #[test]
    fn clear_resets_strikes() {
        let mut tracker = EntryBlameTracker::default();

        assert!(!tracker.record_failure("entry-a", false));
        tracker.clear();
        assert!(!tracker.record_failure("entry-a", false));
    }
}
