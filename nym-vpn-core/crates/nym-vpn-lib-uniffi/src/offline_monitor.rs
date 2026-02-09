// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::sync::Arc;

#[cfg(target_os = "android")]
use crate::android_connectivity_monitor::{
    AndroidConnectivityMonitor, ConnectivityObserverInvalidation, ConnectivityReceiver,
    ConnectivitySender,
};
use nym_offline_monitor::ConnectivityHandle;

#[derive(Debug, uniffi::Object)]
pub struct NymOfflineMonitor {
    connectivity_handle: ConnectivityHandle,
}

#[cfg(target_os = "android")]
#[uniffi::export(async_runtime = "tokio")]
impl NymOfflineMonitor {
    #[uniffi::constructor]
    pub async fn new(connectivity_monitor: Arc<dyn AndroidConnectivityMonitor>) -> Self {
        let connectivity_receiver = register_connectivity_observer(connectivity_monitor);
        let connectivity_handle = nym_offline_monitor::spawn_monitor(connectivity_receiver).await;

        Self {
            connectivity_handle,
        }
    }
}

#[cfg(target_os = "ios")]
#[uniffi::export(async_runtime = "tokio")]
impl NymOfflineMonitor {
    #[uniffi::constructor]
    pub async fn new() -> Self {
        let connectivity_handle = nym_offline_monitor::spawn_monitor().await;
        Self {
            connectivity_handle,
        }
    }
}

impl NymOfflineMonitor {
    pub fn inner(&self) -> ConnectivityHandle {
        self.connectivity_handle.clone()
    }
}

#[cfg(target_os = "android")]
pub fn register_connectivity_observer(
    connectivity_monitor: Arc<dyn AndroidConnectivityMonitor>,
) -> ConnectivityReceiver {
    let (connectivity_tx, connectivity_rx) = tokio::sync::mpsc::unbounded_channel();
    let connectivity_sender = Arc::new(ConnectivitySender::new(connectivity_tx));
    let connectivity_observer_invalidation = ConnectivityObserverInvalidation::new(
        connectivity_monitor.clone(),
        Arc::downgrade(&connectivity_sender),
    );
    let connectivity_receiver =
        ConnectivityReceiver::new(connectivity_rx, connectivity_observer_invalidation);

    connectivity_monitor.add_connectivity_observer(connectivity_sender);
    connectivity_receiver
}
