// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! A process-wide "the device just woke up" signal.
//!
//! The platform wake event (`PacketTunnelProvider.wake()` on Apple) arrives in Swift, outside the
//! tunnel state machine entirely. This lets that event reach whatever in the core wants to react
//! to it - today, the bridge connection's own reconnect loop - without threading it through the
//! tunnel command or state machine layers, neither of which this is a command for.

use std::sync::{Arc, OnceLock};

use tokio::sync::Notify;

static WAKE: OnceLock<Arc<Notify>> = OnceLock::new();

fn wake_notify() -> &'static Arc<Notify> {
    WAKE.get_or_init(|| Arc::new(Notify::new()))
}

/// Call when the platform reports the device has woken up.
pub fn notify_woke() {
    wake_notify().notify_one();
}

/// A clone of the process-wide wake signal, for a task that wants to react to it.
pub fn subscribe() -> Arc<Notify> {
    wake_notify().clone()
}
