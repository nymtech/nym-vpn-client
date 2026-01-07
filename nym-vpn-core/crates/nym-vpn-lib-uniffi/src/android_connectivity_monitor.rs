// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt::Debug,
    sync::{
        Arc, Weak,
        atomic::{AtomicU64, Ordering},
    },
};

/// Unique sender id used to distinguish between multiple instances of `ConnectivitySender` when used on Android side.
static CONNECTIVITY_SENDER_ID: AtomicU64 = AtomicU64::new(0);

/// Abstract network connectivity observer.
#[uniffi::export(with_foreign)]
pub trait ConnectivityObserver: Send + Sync + Debug {
    /// Returns a unique identifier for this observer.
    fn id(&self) -> u64;

    /// Called when the network connectivity status changes.
    fn on_network_change(&self, is_online: bool);
}

/// Abstract network connectivity monitor.
#[uniffi::export(with_foreign)]
pub trait AndroidConnectivityMonitor: Send + Sync + Debug {
    /// Add network connectivity observer.
    fn add_connectivity_observer(&self, observer: Arc<dyn ConnectivityObserver>);

    /// Remove network connectivity observer.
    fn remove_connectivity_observer(&self, observer: Arc<dyn ConnectivityObserver>);
}

/// Network connectivity status sender compatible with uniffi `AndroidTunProvider`
#[derive(Debug)]
pub struct ConnectivitySender {
    id: u64,
    tx: tokio::sync::mpsc::UnboundedSender<bool>,
}

impl ConnectivitySender {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<bool>) -> Self {
        let id = CONNECTIVITY_SENDER_ID.fetch_add(1, Ordering::Relaxed);
        Self { id, tx }
    }
}

impl ConnectivityObserver for ConnectivitySender {
    fn id(&self) -> u64 {
        self.id
    }

    fn on_network_change(&self, is_online: bool) {
        if let Err(e) = self.tx.send(is_online) {
            tracing::warn!("Failed to send network change: {}", e);
        }
    }
}

/// Network connectivity status receiver compatible with `nym_offline_monitor`
#[derive(Debug)]
pub struct ConnectivityReceiver {
    rx: tokio::sync::mpsc::UnboundedReceiver<bool>,
    /// Invalidation token that should be held while interested in network status changes
    _invalidation: ConnectivityObserverInvalidation,
}

impl ConnectivityReceiver {
    pub fn new(
        rx: tokio::sync::mpsc::UnboundedReceiver<bool>,
        _invalidation: ConnectivityObserverInvalidation,
    ) -> Self {
        Self { rx, _invalidation }
    }
}

#[async_trait::async_trait]
impl nym_offline_monitor::NativeConnectivityAdapter for ConnectivityReceiver {
    async fn next_connectivity(&mut self) -> Option<bool> {
        self.rx.recv().await
    }
}

/// Invalidation token cancelling `ConnectivitySender` observation on drop
#[derive(Debug)]
pub struct ConnectivityObserverInvalidation {
    connectivity_monitor: Arc<dyn AndroidConnectivityMonitor>,
    sender: Weak<ConnectivitySender>,
}

impl ConnectivityObserverInvalidation {
    pub fn new(
        connectivity_monitor: Arc<dyn AndroidConnectivityMonitor>,
        sender: Weak<ConnectivitySender>,
    ) -> Self {
        Self {
            connectivity_monitor,
            sender,
        }
    }
}

impl Drop for ConnectivityObserverInvalidation {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.upgrade() {
            self.connectivity_monitor
                .remove_connectivity_observer(sender);
        }
    }
}
