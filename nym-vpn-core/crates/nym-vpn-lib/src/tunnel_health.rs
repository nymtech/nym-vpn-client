// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Shared signals used to correlate ICMP health probes with in-tunnel metadata paths.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Grace window: bandwidth checks default to ~30s; allow slack for probe failures.
pub const METADATA_PATH_HEALTH_GRACE: Duration = Duration::from_secs(45);

#[derive(Clone, Debug, Default)]
pub struct MetadataPathHealth {
    last_success_unix_secs: Arc<AtomicU64>,
}

impl MetadataPathHealth {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_success(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_success_unix_secs.store(now, Ordering::Relaxed);
    }

    pub fn is_recently_healthy(&self, max_age: Duration) -> bool {
        let last = self.last_success_unix_secs.load(Ordering::Relaxed);
        if last == 0 {
            return false;
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(last) <= max_age.as_secs()
    }
}

/// Returns true when an ICMP probe failure should not tear the tunnel down yet.
pub fn should_defer_probe_teardown(
    uses_metadata_endpoint: bool,
    health: Option<&MetadataPathHealth>,
    grace: Duration,
) -> bool {
    uses_metadata_endpoint && health.is_some_and(|h| h.is_recently_healthy(grace))
}

/// Record metadata-path health only when both entry and exit bandwidth checks succeeded.
pub fn record_metadata_path_if_both_legs_ok(
    health: &Option<MetadataPathHealth>,
    entry_ok: bool,
    exit_ok: bool,
) {
    if entry_ok && exit_ok {
        if let Some(health) = health {
            health.record_success();
        }
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
            METADATA_PATH_HEALTH_GRACE
        ));
    }

    #[test]
    fn teardown_when_metadata_stale_or_missing() {
        let health = MetadataPathHealth::new();
        assert!(!should_defer_probe_teardown(
            true,
            Some(&health),
            METADATA_PATH_HEALTH_GRACE
        ));
        assert!(!should_defer_probe_teardown(
            false,
            Some(&health),
            METADATA_PATH_HEALTH_GRACE
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
