// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use tokio::sync::{mpsc, watch};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use nym_apple_dispatch::{Queue, QueueAttr};
use nym_apple_network::{Path as NWPath, PathMonitor, PathStatus};

use super::Connectivity;

/// Maximum duration to wait for the initial state.
const INITIAL_STATE_WAIT: Duration = Duration::from_secs(1);

/// Delay before acting on default route changes.
const DEFAULT_PATH_DEBOUNCE: Duration = Duration::from_millis(250);

pub struct MonitorHandle {
    path_monitor: PathMonitor,
    rx: watch::Receiver<Connectivity>,
}

impl MonitorHandle {
    fn new(path_monitor: PathMonitor, rx: watch::Receiver<Connectivity>) -> Self {
        MonitorHandle { path_monitor, rx }
    }

    pub async fn connectivity(&self) -> Connectivity {
        *self.rx.borrow()
    }

    pub async fn monitor(&mut self) -> Option<Connectivity> {
        self.rx
            .changed()
            .await
            .map(|_| *self.rx.borrow_and_update())
            .ok()
    }
}

pub async fn spawn_monitor() -> Result<MonitorHandle> {
    let (network_path_tx, mut network_path_rx) = mpsc::unbounded_channel();

    // Start system path monitor.
    let path_monitor = start_path_monitor(network_path_tx)?;

    // Wait for initial state
    // Path monitor should always send an update on start(), but if it doesn't then presume the device is online.
    let initial_connectivity = tokio::time::timeout(INITIAL_STATE_WAIT, network_path_rx.recv())
        .await
        .inspect_err(|_| {
            tracing::warn!(
                "Timed out receiving initial update from network monitor. Default to presuming being online."
            );
        })
        .ok()
        .flatten()
        .as_ref()
        .map(map_network_path_to_connectivity)
        .unwrap_or(Connectivity::PresumeOnline);

    let (connectivity_tx, connectivity_rx) = watch::channel(initial_connectivity);
    tracing::debug!("Initial state: {:?}", *connectivity_rx.borrow());

    // Create a task to debounce and broadcast changes outside
    _ = tokio::spawn(async move {
        let mut network_path_rx = debounced::debounced(
            UnboundedReceiverStream::new(network_path_rx),
            DEFAULT_PATH_DEBOUNCE,
        );

        while let Some(network_path) = network_path_rx.next().await {
            let connectivity = map_network_path_to_connectivity(&network_path);
            tracing::trace!("New connectivity: {:?}", connectivity);
            _ = connectivity_tx.send(connectivity);
        }

        tracing::debug!("Connectivity broadcast loop is exiting.");
    });

    Ok(MonitorHandle::new(path_monitor, connectivity_rx))
}

fn start_path_monitor(path_tx: mpsc::UnboundedSender<NWPath>) -> Result<PathMonitor> {
    let queue = Queue::new(Some("net.nymtech.vpn.offline-monitor"), QueueAttr::serial())
        .map_err(Error::CreateDispatchQueue)?;

    // Create and configure path monitor
    let mut path_monitor = PathMonitor::new();
    path_monitor.set_dispatch_queue(&queue);
    path_monitor.set_update_handler(move |nw_path| {
        tracing::trace!("Path status update: {:?}", nw_path);

        if let Err(e) = path_tx.send(nw_path) {
            tracing::warn!("Failed to send new connectivity status: {}", e);
        }
    });
    path_monitor.start();

    Ok(path_monitor)
}

fn map_network_path_to_connectivity(nw_path: &NWPath) -> Connectivity {
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

pub type Result<T, E = Error> = std::result::Result<T, E>;
