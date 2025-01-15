// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use tokio::sync::{watch, Mutex};
use tokio_util::sync::CancellationToken;

use super::Connectivity;

/// Maximum duration to wait for the initial state from path monitor.
const INITIAL_STATE_TIMEOUT: Duration = Duration::from_secs(1);

pub struct MonitorHandle {
    state: Arc<Mutex<Connectivity>>,
}

impl MonitorHandle {
    fn new(state: Arc<Mutex<Connectivity>>) -> Self {
        Self { state }
    }

    pub async fn connectivity(&self) -> Connectivity {
        *self.state.lock().await
    }
}

#[async_trait::async_trait]
pub trait NativeConnectivityAdapter: Send + Sync {
    /// Wait for the next connectivity update.
    ///
    /// - Returns `None` when the event stream is exhausted.
    /// - Returns `Some(true)` if connectivity is online, `Some(false)` if offline.
    ///
    /// # Cancel safety
    ///
    /// This method must guarantee cancel safety.
    async fn next_connectivity(&mut self) -> Option<bool>;
}

#[derive(Debug)]
pub struct Error;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generic error")
    }
}

#[allow(clippy::unused_async)]
pub async fn spawn_monitor(
    sender: watch::Sender<Connectivity>,
    mut connectivity_adapter: impl NativeConnectivityAdapter + 'static,
    shutdown_token: CancellationToken,
) -> Result<MonitorHandle, Error> {
    // Wait for initial state since path monitor should always send an update on start()
    let initial_connectivity = tokio::time::timeout(INITIAL_STATE_TIMEOUT, connectivity_adapter.next_connectivity())
        .await
        .inspect_err(|_| {
            tracing::warn!("Timed out receiving initial update from connectivity adapter. Default to presuming being online.");
        })
        .ok()
        .flatten()
        .map(|connected| {
            Connectivity::Status { connected }
        })
        .unwrap_or(Connectivity::PresumeOnline);

    tracing::info!("Initial connectivity: {:?}", initial_connectivity);

    let state = Arc::new(Mutex::new(initial_connectivity));
    let shared_state = state.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => {
                    break;
                }
                Some(connected) = connectivity_adapter.next_connectivity() => {
                    let mut state_guard = shared_state.lock().await;

                    let connectivity = Connectivity::Status { connected };
                    if *state_guard != connectivity {
                        *state_guard = connectivity;
                        tracing::info!("Connectivity changed: {:?}", connectivity);
                        if sender.send(connectivity).is_err() {
                            break;
                        }
                    }
                }
                else => {
                    break;
                }
            }
        }

        tracing::debug!("Offline monitor exiting");
    });

    Ok(MonitorHandle::new(state))
}
