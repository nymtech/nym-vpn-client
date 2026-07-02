// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Shared signals used to correlate ICMP health probes with in-tunnel metadata paths.

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Grace window: bandwidth checks default to ~30s; allow slack for probe failures.
pub const METADATA_PATH_HEALTH_GRACE: Duration = Duration::from_secs(45);

/// After this many consecutive deferred probe failures, tear down even if metadata was recent.
pub const MAX_CONSECUTIVE_DEFERRED_PROBE_FAILURES: u32 = 3;

#[derive(Clone, Debug, Default)]
pub struct MetadataPathHealth {
    last_success: Arc<Mutex<Option<Instant>>>,
}

impl MetadataPathHealth {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_success(&self) {
        Self::update_last_success(&self.last_success, Instant::now());
    }

    pub fn is_recently_healthy(&self, max_age: Duration) -> bool {
        Self::read_last_success(&self.last_success)
            .is_some_and(|instant| instant.elapsed() <= max_age)
    }

    fn update_last_success(last_success: &Arc<Mutex<Option<Instant>>>, instant: Instant) {
        match last_success.lock() {
            Ok(mut guard) => *guard = Some(instant),
            Err(poisoned) => *poisoned.into_inner() = Some(instant),
        }
    }

    fn read_last_success(last_success: &Arc<Mutex<Option<Instant>>>) -> Option<Instant> {
        match last_success.lock() {
            Ok(guard) => *guard,
            Err(poisoned) => *poisoned.into_inner(),
        }
    }
}

/// Returns true when an ICMP probe failure should not tear the tunnel down yet.
pub fn should_defer_probe_teardown(
    uses_metadata_endpoint: bool,
    health: Option<&MetadataPathHealth>,
    grace: Duration,
    consecutive_deferred_failures: u32,
) -> bool {
    consecutive_deferred_failures < MAX_CONSECUTIVE_DEFERRED_PROBE_FAILURES
        && uses_metadata_endpoint
        && health.is_some_and(|h| h.is_recently_healthy(grace))
}

/// Record metadata-path health only when both entry and exit bandwidth checks succeeded.
pub fn record_metadata_path_if_both_legs_ok(
    health: &Option<MetadataPathHealth>,
    entry_ok: bool,
    exit_ok: bool,
) {
    if entry_ok && exit_ok && let Some(health) = health {
        health.record_success();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unhealthy_before_any_success() {
        let health = MetadataPathHealth::new();
        assert!(!health.is_recently_healthy(METADATA_PATH_HEALTH_GRACE));
    }

    #[test]
    fn healthy_after_success() {
        let health = MetadataPathHealth::new();
        health.record_success();
        assert!(health.is_recently_healthy(METADATA_PATH_HEALTH_GRACE));
    }

    #[test]
    fn defer_teardown_when_metadata_recently_healthy() {
        let health = MetadataPathHealth::new();
        health.record_success();
        assert!(should_defer_probe_teardown(
            true,
            Some(&health),
            METADATA_PATH_HEALTH_GRACE,
            0,
        ));
    }

    #[test]
    fn teardown_when_defer_cap_reached() {
        let health = MetadataPathHealth::new();
        health.record_success();
        assert!(!should_defer_probe_teardown(
            true,
            Some(&health),
            METADATA_PATH_HEALTH_GRACE,
            MAX_CONSECUTIVE_DEFERRED_PROBE_FAILURES,
        ));
    }

    #[test]
    fn teardown_when_metadata_stale_or_missing() {
        let health = MetadataPathHealth::new();
        assert!(!should_defer_probe_teardown(
            true,
            Some(&health),
            METADATA_PATH_HEALTH_GRACE,
            0,
        ));
        assert!(!should_defer_probe_teardown(
            false,
            Some(&health),
            METADATA_PATH_HEALTH_GRACE,
            0,
        ));
    }

    #[test]
    fn record_metadata_only_when_both_legs_ok() {
        let health = MetadataPathHealth::new();
        record_metadata_path_if_both_legs_ok(&Some(health.clone()), true, false);
        assert!(!health.is_recently_healthy(METADATA_PATH_HEALTH_GRACE));
        record_metadata_path_if_both_legs_ok(&Some(health.clone()), true, true);
        assert!(health.is_recently_healthy(METADATA_PATH_HEALTH_GRACE));
    }
}
