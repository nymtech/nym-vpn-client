// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Health tracking for the upstream DNS forwarder.
//!
//! The forwarding resolver keeps pooled connections (e.g. DoH/DoT streams) that go
//! stale when the system sleeps or the network is reconfigured, producing bursts of
//! failed lookups. [`UpstreamHealth`] counts consecutive upstream failures so the
//! resolver can be rebuilt with fresh connections, and [`LogThrottle`] keeps those
//! bursts from flooding the log with identical errors.

use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

/// Number of consecutive upstream failures after which the forwarding resolver is
/// rebuilt to replace potentially stale pooled connections.
const CONSECUTIVE_FAILURES_BEFORE_REBUILD: u32 = 5;

/// Tracks consecutive upstream lookup failures and requests a resolver rebuild once
/// they accumulate. Safe to share between concurrent lookup tasks.
#[derive(Debug, Default)]
pub struct UpstreamHealth {
    consecutive_failures: AtomicU32,
    rebuild_requested: AtomicBool,
}

impl UpstreamHealth {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a successful upstream interaction, resetting the failure streak.
    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    /// Records a failed upstream interaction.
    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= CONSECUTIVE_FAILURES_BEFORE_REBUILD {
            self.consecutive_failures.store(0, Ordering::Relaxed);
            self.rebuild_requested.store(true, Ordering::Relaxed);
        }
    }

    /// Returns true when enough consecutive failures accumulated since the last
    /// request. The request is consumed: subsequent calls return false until another
    /// full streak of failures accumulates.
    pub fn take_rebuild_request(&self) -> bool {
        self.rebuild_requested.swap(false, Ordering::Relaxed)
    }
}

/// Rate limiter for repetitive log statements: allows one log line per window and
/// counts how many were suppressed in between.
#[derive(Debug)]
pub struct LogThrottle {
    window: Duration,
    state: Mutex<ThrottleState>,
}

#[derive(Debug, Default)]
struct ThrottleState {
    last_logged: Option<Instant>,
    suppressed: u64,
}

impl LogThrottle {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            state: Mutex::new(ThrottleState::default()),
        }
    }

    /// Returns `Some(suppressed)` when the caller should log now, where `suppressed`
    /// is the number of occurrences swallowed since the last logged one; returns
    /// `None` when the statement should be suppressed.
    pub fn try_log(&self, now: Instant) -> Option<u64> {
        let mut state = self.state.lock().unwrap();
        match state.last_logged {
            Some(last_logged) if now.duration_since(last_logged) < self.window => {
                state.suppressed += 1;
                None
            }
            _ => {
                let suppressed = state.suppressed;
                state.last_logged = Some(now);
                state.suppressed = 0;
                Some(suppressed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_rebuild_below_failure_threshold() {
        let health = UpstreamHealth::new();
        for _ in 0..CONSECUTIVE_FAILURES_BEFORE_REBUILD - 1 {
            health.record_failure();
        }
        assert!(!health.take_rebuild_request());
    }

    #[test]
    fn rebuild_requested_after_consecutive_failures() {
        let health = UpstreamHealth::new();
        for _ in 0..CONSECUTIVE_FAILURES_BEFORE_REBUILD {
            health.record_failure();
        }
        assert!(health.take_rebuild_request());
        assert!(
            !health.take_rebuild_request(),
            "the rebuild request must be consumed by taking it"
        );
    }

    #[test]
    fn success_resets_failure_streak() {
        let health = UpstreamHealth::new();
        for _ in 0..CONSECUTIVE_FAILURES_BEFORE_REBUILD - 1 {
            health.record_failure();
        }
        health.record_success();
        for _ in 0..CONSECUTIVE_FAILURES_BEFORE_REBUILD - 1 {
            health.record_failure();
        }
        assert!(!health.take_rebuild_request());
    }

    #[test]
    fn another_full_streak_is_required_after_a_rebuild_request() {
        let health = UpstreamHealth::new();
        for _ in 0..CONSECUTIVE_FAILURES_BEFORE_REBUILD {
            health.record_failure();
        }
        assert!(health.take_rebuild_request());

        health.record_failure();
        assert!(!health.take_rebuild_request());
    }

    #[test]
    fn throttle_allows_first_log() {
        let throttle = LogThrottle::new(Duration::from_secs(5));
        assert_eq!(throttle.try_log(Instant::now()), Some(0));
    }

    #[test]
    fn throttle_suppresses_within_window() {
        let throttle = LogThrottle::new(Duration::from_secs(5));
        let start = Instant::now();
        assert_eq!(throttle.try_log(start), Some(0));
        assert_eq!(throttle.try_log(start + Duration::from_secs(1)), None);
        assert_eq!(throttle.try_log(start + Duration::from_secs(2)), None);
    }

    #[test]
    fn throttle_logs_again_after_window_with_suppressed_count() {
        let throttle = LogThrottle::new(Duration::from_secs(5));
        let start = Instant::now();
        assert_eq!(throttle.try_log(start), Some(0));
        assert_eq!(throttle.try_log(start + Duration::from_secs(1)), None);
        assert_eq!(throttle.try_log(start + Duration::from_secs(2)), None);
        assert_eq!(throttle.try_log(start + Duration::from_secs(6)), Some(2));
    }

    #[test]
    fn throttle_resets_suppressed_count_after_logging() {
        let throttle = LogThrottle::new(Duration::from_secs(5));
        let start = Instant::now();
        assert_eq!(throttle.try_log(start), Some(0));
        assert_eq!(throttle.try_log(start + Duration::from_secs(1)), None);
        assert_eq!(throttle.try_log(start + Duration::from_secs(6)), Some(1));
        assert_eq!(throttle.try_log(start + Duration::from_secs(12)), Some(0));
    }
}
