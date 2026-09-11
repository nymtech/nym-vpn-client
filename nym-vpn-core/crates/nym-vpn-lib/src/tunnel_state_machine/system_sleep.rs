// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detection of system sleep intervals overlapping an operation.
//!
//! The monotonic clock pauses while the system is asleep whereas the wall clock does
//! not, so a wall-vs-monotonic divergence since a marker was created reveals that the
//! system slept in between. On platforms where the monotonic clock keeps counting
//! through sleep the clocks never diverge and detection simply reports no sleep.

use std::time::{Duration, Instant, SystemTime};

/// Minimum wall-vs-monotonic divergence interpreted as a system sleep interval,
/// large enough to not be confused with NTP clock adjustments.
const SLEEP_DETECTION_THRESHOLD: Duration = Duration::from_secs(10);

/// A point in time captured on both the wall and monotonic clocks.
#[derive(Debug, Clone, Copy)]
pub struct SleepMarker {
    wall: SystemTime,
    monotonic: Instant,
}

impl SleepMarker {
    pub fn now() -> Self {
        Self {
            wall: SystemTime::now(),
            monotonic: Instant::now(),
        }
    }

    /// Returns true when the system appears to have slept since the marker was created.
    pub fn system_slept(&self) -> bool {
        system_slept(
            self.wall.elapsed().unwrap_or_default(),
            self.monotonic.elapsed(),
        )
    }
}

fn system_slept(wall_elapsed: Duration, monotonic_elapsed: Duration) -> bool {
    wall_elapsed.saturating_sub(monotonic_elapsed) >= SLEEP_DETECTION_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_sleep_when_clocks_agree() {
        assert!(!system_slept(
            Duration::from_secs(60),
            Duration::from_secs(60)
        ));
    }

    #[test]
    fn no_sleep_on_small_clock_jitter() {
        assert!(!system_slept(
            Duration::from_secs(63),
            Duration::from_secs(60)
        ));
    }

    #[test]
    fn sleep_detected_when_wall_clock_ran_ahead() {
        // Observed on macOS: 15m56s of wall time vs 11s of monotonic time across a nap.
        assert!(system_slept(
            Duration::from_secs(956),
            Duration::from_secs(11)
        ));
    }

    #[test]
    fn no_sleep_when_wall_clock_moved_backwards() {
        assert!(!system_slept(
            Duration::from_secs(10),
            Duration::from_secs(60)
        ));
    }
}
