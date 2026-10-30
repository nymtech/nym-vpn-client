// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use tracing::debug;

/// RAII guard to ensure connection counter is decremented when dropped
pub struct ConnectionGuard {
    active_connections: Arc<AtomicU32>,
}

impl ConnectionGuard {
    pub fn new(active_connections: Arc<AtomicU32>) -> Self {
        let count = active_connections.fetch_add(1, Ordering::Relaxed) + 1;
        debug!("Active connections: {}", count);
        Self { active_connections }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let count = self.active_connections.fetch_sub(1, Ordering::Relaxed) - 1;
        debug!("Active connections: {}", count);
    }
}
