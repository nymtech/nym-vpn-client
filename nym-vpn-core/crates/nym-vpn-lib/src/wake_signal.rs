// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! A process-wide "the device just woke up" signal.
//!
//! The platform wake event (`PacketTunnelProvider.wake()` on Apple) arrives in Swift, outside the
//! tunnel state machine entirely. This lets that event reach whatever in the core wants to react
//! to it - today, the bridge connection's own reconnect loop - without threading it through the
//! tunnel command or state machine layers, neither of which this is a command for.

use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime};

use tokio::sync::Notify;

static WAKE: OnceLock<Arc<Notify>> = OnceLock::new();
static LAST_WOKE: Mutex<Option<SystemTime>> = Mutex::new(None);

// iOS calls `wake()` on brief process thaw/resuspend cycles while the screen stays off, not
// only on a real device wake - observed firing in bursts every few seconds for minutes at a
// stretch. Collapse those into one redial per window instead of tearing the bridge down repeatedly.
const DEBOUNCE: Duration = Duration::from_secs(5);

fn wake_notify() -> &'static Arc<Notify> {
    WAKE.get_or_init(|| Arc::new(Notify::new()))
}

/// Call when the platform reports the device has woken up. Debounced - see `DEBOUNCE`.
///
/// Uses wall-clock time, not `Instant`: on Apple targets `Instant` is `CLOCK_UPTIME_RAW`, and
/// everywhere else it's `CLOCK_MONOTONIC` - both stop advancing while the device is asleep, so a
/// real multi-minute sleep would otherwise look like under a second had passed and get wrongly
/// debounced away, exactly the wake this feature exists to catch.
pub fn notify_woke() {
    let mut last = LAST_WOKE.lock().unwrap();
    let now = SystemTime::now();
    if last.is_some_and(|t| now.duration_since(t).is_ok_and(|d| d < DEBOUNCE)) {
        return;
    }
    *last = Some(now);
    wake_notify().notify_one();
}

/// A clone of the process-wide wake signal, for a task that wants to react to it.
pub fn subscribe() -> Arc<Notify> {
    wake_notify().clone()
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn second_wake_within_debounce_window_is_suppressed() {
        let wake = subscribe();

        notify_woke();
        // Drains whatever permit is pending (this module's statics are process-wide, so a
        // prior test run may have already left one) before asserting on fresh calls below.
        tokio::time::timeout(Duration::from_millis(200), wake.notified())
            .await
            .expect("notify_woke should have signalled a waiter");

        notify_woke();
        assert!(
            tokio::time::timeout(Duration::from_millis(200), wake.notified())
                .await
                .is_err(),
            "a second wake within the debounce window should not signal again"
        );
    }
}
