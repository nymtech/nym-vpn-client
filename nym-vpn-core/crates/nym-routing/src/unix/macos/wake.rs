// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use naptime::{EventHandler, Naptime};
use tokio::sync::mpsc::UnboundedSender;

/// Observer monitoring computer wake events
pub struct OSWakeObserver {
    _inner: Naptime,
}

impl OSWakeObserver {
    /// Register for receiving wake events over the `tx` channel.
    pub fn register(tx: UnboundedSender<()>) -> std::io::Result<Self> {
        let event_handler = PowerEventHandler::new(tx);
        let nt = Naptime::new(event_handler).map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(Self { _inner: nt })
    }
}

struct PowerEventHandler {
    tx: UnboundedSender<()>,
}

impl PowerEventHandler {
    pub fn new(tx: UnboundedSender<()>) -> Self {
        Self { tx }
    }
}

impl EventHandler for PowerEventHandler {
    fn sleep_query(&mut self) -> naptime::SleepQueryResponse {
        tracing::info!("Allow sleep to proceed");
        naptime::SleepQueryResponse::Allow
    }

    fn sleep_failed(&mut self) {
        tracing::info!("Sleep failed");
    }

    fn sleep(&mut self) {
        tracing::info!("Computer is entering sleep");
    }

    fn wake(&mut self) {
        tracing::info!("Computer is awake");
        if let Err(e) = self.tx.send(()) {
            tracing::error!("Failed to send wake event: {e}");
        }
    }
}
