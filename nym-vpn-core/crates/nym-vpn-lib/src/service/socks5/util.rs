// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::atomic::{AtomicU32, Ordering};
use tracing::debug;

/// RAII guard to ensure connection counter is decremented when dropped
pub struct ConnectionGuard<'a> {
    active_connections: &'a AtomicU32,
}

impl<'a> ConnectionGuard<'a> {
    pub fn new(active_connections: &'a AtomicU32) -> Self {
        let count = active_connections.fetch_add(1, Ordering::Relaxed) + 1;
        debug!("Active connections: {}", count);
        Self { active_connections }
    }
}

impl Drop for ConnectionGuard<'_> {
    fn drop(&mut self) {
        let count = self.active_connections.fetch_sub(1, Ordering::Relaxed) - 1;
        debug!("Active connections: {}", count);
    }
}
