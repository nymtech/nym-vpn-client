// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_offline_monitor::NativeConnectivityAdapter;
use tokio::sync::mpsc;

use crate::tunnel_provider::android::{AndroidTunProvider, ConnectivityObserver};

/// Adapter bridging offline detection with native Android Connectivity Manager via uniffi.
pub struct AndroidConnectivityAdapter {
    rx: mpsc::UnboundedReceiver<bool>,
    /// Inner connection observer that's being registered with `tun_provider`
    inner: Option<Arc<InnerConnectionObserver>>,
    /// Reference to `tun_provider` used for unregistering the observer on drop.
    tun_provider: Arc<dyn AndroidTunProvider>,
}

impl AndroidConnectivityAdapter {
    /// Create new connectivity adapter registering it with `tun_provider.`
    /// Automatically unregisters the observer on drop.
    pub fn new(tun_provider: Arc<dyn AndroidTunProvider>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let inner = Arc::new(InnerConnectionObserver { tx });
        tun_provider.add_connectivity_observer(inner.clone());
        Self {
            rx,
            inner: Some(inner),
            tun_provider,
        }
    }
}

impl Drop for AndroidConnectivityAdapter {
    fn drop(&mut self) {
        self.inner
            .take()
            .map(|observer| self.tun_provider.remove_connectivity_observer(observer));
    }
}

#[async_trait::async_trait]
impl NativeConnectivityAdapter for AndroidConnectivityAdapter {
    async fn next_connectivity(&mut self) -> Option<bool> {
        self.rx.recv().await
    }
}

#[derive(Debug)]
struct InnerConnectionObserver {
    tx: mpsc::UnboundedSender<bool>,
}

impl ConnectivityObserver for InnerConnectionObserver {
    fn on_network_change(&self, is_online: bool) {
        if let Err(e) = self.tx.send(is_online) {
            tracing::warn!("Failed to send network change: {}", e);
        }
    }
}
