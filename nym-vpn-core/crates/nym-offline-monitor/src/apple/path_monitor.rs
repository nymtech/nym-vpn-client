// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use tokio::sync::{mpsc, watch, Mutex};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tokio_util::sync::CancellationToken;

use nym_apple_dispatch::{Queue, QueueAttr};
use nym_apple_network::{InterfaceType, Path, PathMonitor, PathStatus};

use crate::Connectivity;

/// Maximum duration to wait for the initial state from path monitor.
const INITIAL_STATE_TIMEOUT: Duration = Duration::from_secs(1);

/// Delay before acting on default route changes.
const DEFAULT_PATH_DEBOUNCE: Duration = Duration::from_secs(1);

pub struct MonitorHandle {
    state: Arc<Mutex<Connectivity>>,

    // Network path monitor.
    // Has to be retained while monitoring path updates. Auto cancels on drop.
    _path_monitor: PathMonitor,
}

impl MonitorHandle {
    fn new(initial_state: Arc<Mutex<Connectivity>>, path_monitor: PathMonitor) -> Self {
        MonitorHandle {
            state: initial_state,
            _path_monitor: path_monitor,
        }
    }

    pub async fn connectivity(&self) -> Connectivity {
        *self.state.lock().await
    }
}

pub async fn spawn_monitor(
    sender: watch::Sender<Connectivity>,
    shutdown_token: CancellationToken,
) -> Result<MonitorHandle, Error> {
    let (network_path_tx, mut network_path_rx) = mpsc::unbounded_channel();
    let path_monitor = start_path_monitor(network_path_tx)?;

    // Wait for initial state since path monitor should always send an update on start()
    let initial_connectivity = tokio::time::timeout(INITIAL_STATE_TIMEOUT, network_path_rx.recv())
        .await
        .inspect_err(|_| {
            tracing::warn!("Timed out receiving initial update from network monitor. Default to presuming being online.");
        })
        .ok()
        .flatten()
        .as_ref()
        .map(map_network_path_to_connectivity)
        .unwrap_or(Connectivity::PresumeOnline);

    tracing::info!("Initial connectivity: {:?}", initial_connectivity);

    let initial_state = Arc::new(Mutex::new(initial_connectivity));
    let shared_state = initial_state.clone();

    _ = tokio::spawn(async move {
        let mut network_path_stream = debounced::debounced(
            UnboundedReceiverStream::new(network_path_rx),
            DEFAULT_PATH_DEBOUNCE,
        );

        loop {
            tokio::select! {
                network_path = network_path_stream.next() => {
                    let Some(network_path) = network_path else {
                        break
                    };

                    tracing::info!("Path update: {}", network_path.description());

                    let mut state_guard = shared_state.lock().await;
                    let connectivity = map_network_path_to_connectivity(&network_path);

                    if *state_guard != connectivity {
                        *state_guard = connectivity;
                        tracing::info!("Connectivity changed: {:?}", connectivity);
                        if sender.send(connectivity).is_err() {
                            break;
                        }
                    }
                },
                _ = shutdown_token.cancelled() => {
                    break;
                }
            }
        }

        tracing::debug!("Offline monitor exiting");
    });

    Ok(MonitorHandle::new(initial_state, path_monitor))
}

fn start_path_monitor(path_tx: mpsc::UnboundedSender<Path>) -> Result<PathMonitor, Error> {
    let queue = Queue::new(Some("net.nymtech.vpn.offline-monitor"), QueueAttr::serial())
        .map_err(Error::CreateDispatchQueue)?;

    let mut path_monitor = PathMonitor::new();
    path_monitor.prohibit_interface_type(InterfaceType::Other);
    path_monitor.set_dispatch_queue(&queue);
    path_monitor.set_update_handler(move |nw_path| {
        if let Err(e) = path_tx.send(nw_path) {
            tracing::warn!("Failed to send new connectivity status: {}", e);
        }
    });
    path_monitor.start();

    Ok(path_monitor)
}

fn map_network_path_to_connectivity(nw_path: &Path) -> Connectivity {
    match nw_path.status() {
        PathStatus::Satisfiable | PathStatus::Satisfied => Connectivity::Status {
            ipv4: nw_path.supports_ipv4(),
            ipv6: nw_path.supports_ipv6(),
        },
        PathStatus::Unsatisfied => Connectivity::Status {
            ipv4: false,
            ipv6: false,
        },
        path_status @ PathStatus::Unknown(_) | path_status @ PathStatus::Invalid => {
            tracing::warn!("Cannot map {:?} path status to connectivity.", path_status);
            Connectivity::PresumeOnline
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create a dispatch queue")]
    CreateDispatchQueue(#[source] std::ffi::NulError),
}
