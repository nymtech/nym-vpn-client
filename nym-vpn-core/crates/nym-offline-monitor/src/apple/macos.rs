// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! This module has been reimplemented multiple times, often to no avail, with main issues being
//! that the app gets stuck in an offline state, blocking all internet access and preventing the
//! user from connecting to a relay.
//!
//! See [RouteManagerHandle::default_route_listener].
//!
//! This offline monitor synthesizes an offline state between network switches and before coming
//! online from an offline state. This is done to work around issues with DNS being blocked due
//! to macOS's connectivity check. In the offline state, a DNS server on localhost prevents the
//! connectivity check from being blocked.

use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use futures::future::{Fuse, FutureExt};
use nym_routing::{DefaultRouteEvent, RouteManagerHandle};
use tokio::sync::{watch, Mutex};
use tokio_util::sync::CancellationToken;

use crate::Connectivity;

use super::path_monitor;

/// Use Apple's path monitor facilities to track offline state.
static USE_PATH_MONITOR: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("NYM_USE_PATH_MONITOR")
        .map(|v| v != "0")
        .unwrap_or(false)
});

const SYNTHETIC_OFFLINE_DURATION: Duration = Duration::from_secs(1);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to initialize route monitor")]
    StartRouteMonitor(#[from] nym_routing::Error),

    #[error("Failed to initialize path monitor")]
    StartPathMonitor(#[from] path_monitor::Error),
}

enum MonitorHandleInner {
    State(Arc<Mutex<ConnectivityInner>>),
    PathMonitorImp(path_monitor::MonitorHandle),
}

pub struct MonitorHandle {
    inner: MonitorHandleInner,
}

impl MonitorHandle {
    fn new(inner: MonitorHandleInner) -> Self {
        Self { inner }
    }

    /// Return whether the host is offline
    pub async fn connectivity(&self) -> Connectivity {
        match &self.inner {
            MonitorHandleInner::State(state) => state.lock().await.into_connectivity(),
            MonitorHandleInner::PathMonitorImp(imp) => imp.connectivity().await,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ConnectivityInner {
    /// Whether IPv4 connectivity seems to be available on the host.
    ipv4: bool,
    /// Whether IPv6 connectivity seems to be available on the host.
    ipv6: bool,
}

impl ConnectivityInner {
    fn into_connectivity(self) -> Connectivity {
        Connectivity::Status {
            ipv4: self.ipv4,
            ipv6: self.ipv6,
        }
    }

    fn is_online(&self) -> bool {
        self.into_connectivity().is_online()
    }
}

pub async fn spawn_monitor(
    notify_tx: watch::Sender<Connectivity>,
    route_manager: RouteManagerHandle,
    shutdown_token: CancellationToken,
) -> Result<MonitorHandle, Error> {
    if *USE_PATH_MONITOR {
        tracing::info!("Using path monitor.");
        Ok(
            super::path_monitor::spawn_monitor(notify_tx, shutdown_token)
                .await
                .map(|imp| MonitorHandle::new(MonitorHandleInner::PathMonitorImp(imp)))?,
        )
    } else {
        spawn_route_monitor(notify_tx, route_manager, shutdown_token).await
    }
}

async fn spawn_route_monitor(
    notify_tx: watch::Sender<Connectivity>,
    route_manager: RouteManagerHandle,
    shutdown_token: CancellationToken,
) -> Result<MonitorHandle, Error> {
    // note: begin observing before initializing the state
    let mut route_listener = route_manager.default_route_listener().await?;

    let (ipv4, ipv6) = match route_manager.get_default_routes().await {
        Ok((v4_route, v6_route)) => (v4_route.is_some(), v6_route.is_some()),
        Err(error) => {
            tracing::warn!("Failed to initialize offline monitor: {error}");
            // Fail open: Assume that we have connectivity if we cannot determine the existence of
            // a default route, since we don't want to block the user from connecting
            (true, true)
        }
    };

    let initial_state = ConnectivityInner { ipv4, ipv6 };
    let mut real_state = initial_state;
    let state = Arc::new(Mutex::new(initial_state));
    let shared_state = state.clone();

    // Detect changes to the default route
    tokio::spawn(async move {
        let mut timeout = Fuse::terminated();

        loop {
            nym_common::detect_flood!();

            tokio::select! {
                _ = &mut timeout => {
                    // Update shared state
                    let mut state = shared_state.lock().await;
                    if real_state.is_online() {
                        tracing::info!("Connectivity changed: Connected");
                        let _ = notify_tx.send(real_state.into_connectivity());
                    }

                    *state = real_state;
                }
                route_event = route_listener.recv() => {
                    let Some(event) = route_event else {
                        break;
                    };

                    // Update real state
                    match event {
                        DefaultRouteEvent::AddedOrChangedV4 => {
                            real_state.ipv4 = true;
                        }
                        DefaultRouteEvent::AddedOrChangedV6 => {
                            real_state.ipv6 = true;
                        }
                        DefaultRouteEvent::RemovedV4 => {
                            real_state.ipv4 = false;
                        }
                        DefaultRouteEvent::RemovedV6 => {
                            real_state.ipv6 = false;
                        }
                    }

                    // Synthesize offline state
                    // Update shared state
                    let mut state = shared_state.lock().await;
                    let previous_connectivity = *state;
                    state.ipv4 = false;
                    state.ipv6 = false;

                    if previous_connectivity.is_online() {
                        let _ = notify_tx.send(state.into_connectivity());
                        tracing::info!("Connectivity changed: Offline");
                    }

                    if real_state.is_online() {
                        timeout = Box::pin(tokio::time::sleep(SYNTHETIC_OFFLINE_DURATION)).fuse();
                    }
                }
                _ = shutdown_token.cancelled() => {
                    break
                }
            }
        }

        tracing::trace!("Offline monitor exiting");
    });

    Ok(MonitorHandle::new(MonitorHandleInner::State(state)))
}
