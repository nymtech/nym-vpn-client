// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{ffi::CStr, fmt, ops::Deref, ptr::NonNull, sync::Arc, time::Duration};

use dispatch2::{DispatchQueue, DispatchQueueAttr};
use objc2::runtime::{AnyObject, NSObject};
use objc2_network::{NWPath, NWPathMonitor, NWRetained, nw_path_status_t};
use tokio::sync::{Mutex, mpsc, watch};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;

use crate::Connectivity;

/// Maximum duration to wait for the initial state from path monitor.
const INITIAL_STATE_TIMEOUT: Duration = Duration::from_secs(1);

/// Delay before acting on default route changes.
const DEFAULT_PATH_DEBOUNCE: Duration = Duration::from_secs(1);

pub struct ConnectivityHandle {
    state: Arc<Mutex<Connectivity>>,

    // Network path monitor.
    // Has to be retained while monitoring path updates. Auto cancels on drop.
    _path_monitor: PathMonitor,
}

impl ConnectivityHandle {
    fn new(initial_state: Arc<Mutex<Connectivity>>, path_monitor: PathMonitor) -> Self {
        ConnectivityHandle {
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
) -> ConnectivityHandle {
    let (network_path_tx, mut network_path_rx) = mpsc::unbounded_channel();
    let path_monitor = start_path_monitor(c"net.nymtech.vpn.offline-monitor", network_path_tx);

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

                    tracing::info!("Path update: {:?}", network_path);

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

    ConnectivityHandle::new(initial_state, path_monitor)
}

/// Network objects that are safe to use from any thread.
///
/// # Safety
///
/// The implementing type must be safe to send and share between threads.
pub unsafe trait ThreadSafeObject {}

// SAFETY: `nw_path_monitor_t` is a reference counted Network object that can be used from any
// thread. Update handlers are invoked on the dispatch queue assigned to the monitor.
unsafe impl ThreadSafeObject for NWPathMonitor {}

// SAFETY: `nw_path_t` is an immutable, reference counted Network object that can be used from
// any thread.
unsafe impl ThreadSafeObject for NWPath {}

/// Retained Network object that can be shared between threads.
pub struct ThreadSafe<T: ThreadSafeObject>(NWRetained<T>);

// SAFETY: Guaranteed by `ThreadSafeObject`.
unsafe impl<T: ThreadSafeObject> Send for ThreadSafe<T> {}
// SAFETY: Guaranteed by `ThreadSafeObject`.
unsafe impl<T: ThreadSafeObject> Sync for ThreadSafe<T> {}

impl<T: ThreadSafeObject> Deref for ThreadSafe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type PathMonitor = ThreadSafe<NWPathMonitor>;
pub type NetworkPath = ThreadSafe<NWPath>;

impl fmt::Debug for NetworkPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let object: &AnyObject = self.as_ref();
        let description = object
            .downcast_ref::<NSObject>()
            .map(|object| format!("{object:?}"))
            .unwrap_or_default();

        f.write_str(&description)
    }
}

/// Starts monitoring network path changes, sending each update to `path_tx`.
///
/// The returned monitor must be retained for as long as updates are needed.
pub fn start_path_monitor(
    queue_label: &CStr,
    path_tx: mpsc::UnboundedSender<NetworkPath>,
) -> PathMonitor {
    let queue = DispatchQueue::new(Some(queue_label), DispatchQueueAttr::SERIAL);

    let path_monitor = NWPathMonitor::new();
    unsafe { path_monitor.set_queue(&queue) };
    let update_handler = block2::RcBlock::new(move |nw_path: NonNull<NWPath>| {
        let path = ThreadSafe(unsafe { NWRetained::retain(nw_path) });

        if let Err(e) = path_tx.send(path) {
            tracing::warn!("Failed to send network path update: {}", e);
        }
    });
    path_monitor.set_update_handler(&update_handler);
    path_monitor.start();

    ThreadSafe(path_monitor)
}

fn map_network_path_to_connectivity(nw_path: &NetworkPath) -> Connectivity {
    let path_status = nw_path.status();

    if path_status == nw_path_status_t::satisfiable || path_status == nw_path_status_t::satisfied {
        // Simulator falsely reports that host network does not support neither ipv4 nor ipv6
        #[cfg(target_env = "sim")]
        {
            Connectivity::PresumeOnline
        }

        #[cfg(not(target_env = "sim"))]
        {
            Connectivity::Status {
                ipv4: nw_path.has_ipv4(),
                ipv6: nw_path.has_ipv6(),
            }
        }
    } else if path_status == nw_path_status_t::unsatisfied {
        Connectivity::Status {
            ipv4: false,
            ipv6: false,
        }
    } else {
        tracing::warn!("Cannot map {path_status:?} path status to connectivity");
        Connectivity::PresumeOnline
    }
}
