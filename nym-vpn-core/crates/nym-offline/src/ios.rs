// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use futures::channel::mpsc::UnboundedSender;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tokio_util::sync::CancellationToken;

use nym_apple_dispatch::{Queue, QueueAttr};
use nym_apple_network::{PathMonitor, PathStatus};

use super::Connectivity;

/// Maximum duration to wait for the initial state.
const INITIAL_STATE_WAIT: Duration = Duration::from_secs(1);

/// Delay before acting on default route changes.
const DEFAULT_PATH_DEBOUNCE: Duration = Duration::from_millis(250);

pub struct MonitorHandle {
    current_state: Arc<Mutex<Connectivity>>,
    path_monitor: PathMonitor,
    broadcaster_task: JoinHandle<()>,
    cancel_token: CancellationToken,
}

impl MonitorHandle {
    fn new(
        current_state: Arc<Mutex<Connectivity>>,
        path_monitor: PathMonitor,
        broadcaster_task: JoinHandle<()>,
        cancel_token: CancellationToken,
    ) -> Self {
        MonitorHandle {
            current_state,
            path_monitor,
            broadcaster_task,
            cancel_token,
        }
    }

    pub async fn connectivity(&self) -> Connectivity {
        *self.current_state.lock().await
    }

    #[allow(unused)]
    pub async fn cancel_and_wait(mut self) {
        self.path_monitor.cancel();
        self.cancel_token.cancel();

        if let Err(e) = self.broadcaster_task.await {
            tracing::error!("Failed to join broadcast task: {}", e);
        }
    }
}

pub async fn spawn_monitor(sender: UnboundedSender<Connectivity>) -> Result<MonitorHandle> {
    let (connectivity_tx, mut connectivity_rx) = mpsc::unbounded_channel();

    // Start system path monitor.
    let path_monitor = start_path_monitor(connectivity_tx)?;

    // Wait for initial state
    // Path monitor should always send an update on start(), but if it doesn't then presume the device is online.
    let initial_state = tokio::time::timeout(INITIAL_STATE_WAIT, connectivity_rx.recv())
        .await
        .ok()
        .flatten()
        .unwrap_or(Connectivity::PresumeOnline);
    tracing::debug!("Initial state: {:?}", initial_state);

    let current_state = Arc::new(Mutex::new(initial_state));
    let cloned_current_state = current_state.clone();

    // Create a task to debouce and broadcast changes outside
    let cancel_token = CancellationToken::new();
    let child_token = cancel_token.child_token();
    let broadcaster_task = tokio::spawn(async move {
        let mut connectivity_rx = debounced::debounced(
            UnboundedReceiverStream::new(connectivity_rx),
            DEFAULT_PATH_DEBOUNCE,
        );

        loop {
            tokio::select! {
                Some(connectivity) = connectivity_rx.next() => {
                    tracing::trace!("New connectivity: {:?}", connectivity);

                    let mut state = cloned_current_state.lock().await;
                    *state = connectivity;

                    _ = sender.unbounded_send(connectivity);
                }
                _ = child_token.cancelled() => {
                    break;
                }
                else => {
                    break;
                }
            }
        }
    });

    Ok(MonitorHandle::new(
        current_state,
        path_monitor,
        broadcaster_task,
        cancel_token,
    ))
}

fn start_path_monitor(connectivity_tx: mpsc::UnboundedSender<Connectivity>) -> Result<PathMonitor> {
    let queue = Queue::new(Some("net.nymtech.vpn.offline-monitor"), QueueAttr::serial())
        .map_err(Error::CreateDispatchQueue)?;

    // Create and configure path monitor
    let mut path_monitor = PathMonitor::new();
    path_monitor.set_dispatch_queue(&queue);
    path_monitor.set_update_handler(move |nw_path| {
        let connectivity = match nw_path.status() {
            PathStatus::Satisfiable | PathStatus::Satisfied => Connectivity::Status {
                ipv4: nw_path.supports_ipv4(),
                ipv6: nw_path.supports_ipv6(),
            },
            PathStatus::Unsatisfied => Connectivity::Status {
                ipv4: false,
                ipv6: false,
            },
            path_status @ PathStatus::Unknown(_) | path_status @ PathStatus::Invalid => {
                tracing::warn!("Received {:?} path status", path_status);
                Connectivity::PresumeOnline
            }
        };
        tracing::trace!("Received path status update: {:?}", nw_path);

        if let Err(e) = connectivity_tx.send(connectivity) {
            tracing::warn!("Failed to send new connectivity status: {}", e);
        }
    });
    path_monitor.start();

    Ok(path_monitor)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create a dispatch queue")]
    CreateDispatchQueue(#[source] std::ffi::NulError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
