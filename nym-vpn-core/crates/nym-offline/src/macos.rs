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

use std::{sync::Arc, time::Duration};

use futures::{
    future::{Fuse, FutureExt},
    StreamExt,
};
use nym_routing::{DefaultRouteEvent, RouteManagerHandle};
use tokio::sync::Mutex;

use super::Connectivity;

const SYNTHETIC_OFFLINE_DURATION: Duration = Duration::from_secs(1);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to initialize route monitor")]
    StartMonitorError(#[from] nym_routing::Error),
}

pub struct MonitorHandle {
    state: Arc<Mutex<ConnectivityInner>>,
    _notify_tx: Arc<mpsc::UnboundedSender<Connectivity>>,
}

impl MonitorHandle {
    /// Return whether the host is offline
    pub async fn connectivity(&self) -> Connectivity {
        self.state.lock().await.into_connectivity()
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
    notify_tx: mpsc::UnboundedSender<Connectivity>,
    route_manager: RouteManagerHandle,
) -> Result<MonitorHandle, Error> {
    let notify_tx = Arc::new(notify_tx);

    // note: begin observing before initializing the state
    let route_listener = route_manager.default_route_listener().await?;

    let (ipv4, ipv6) = match route_manager.get_default_routes().await {
        Ok((v4_route, v6_route)) => (v4_route.is_some(), v6_route.is_some()),
        Err(error) => {
            tracing::warn!("Failed to initialize offline monitor: {error}");
            // Fail open: Assume that we have connectivity if we cannot determine the existence of
            // a default route, since we don't want to block the user from connecting
            (true, true)
        }
    };

    let state = ConnectivityInner { ipv4, ipv6 };
    let mut real_state = state;

    let state = Arc::new(Mutex::new(state));

    let weak_state = Arc::downgrade(&state);
    let weak_notify_tx = Arc::downgrade(&notify_tx);

    // Detect changes to the default route
    tokio::spawn(async move {
        let mut timeout = Fuse::terminated();
        let mut route_listener = route_listener.fuse();

        loop {
            nym_common::detect_flood!();

            tokio::select! {
                _ = &mut timeout => {
                    // Update shared state
                    let Some(state) = weak_state.upgrade() else {
                        break;
                    };

                    let mut state = state.lock().await;
                    if real_state.is_online() {
                        tracing::info!("Connectivity changed: Connected");
                        let Some(tx) = weak_notify_tx.upgrade() else {
                            break;
                        };
                        let _ = tx.send(real_state.into_connectivity());
                    }

                    *state = real_state;
                }

                route_event = route_listener.next() => {
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
                    let Some(state) = weak_state.upgrade() else {
                        break;
                    };
                    let mut state = state.lock().await;
                    let previous_connectivity = *state;
                    state.ipv4 = false;
                    state.ipv6 = false;

                    if previous_connectivity.is_online() {
                        let Some(tx) = weak_notify_tx.upgrade() else {
                            break;
                        };
                        let _ = tx.send(state.into_connectivity());
                        tracing::info!("Connectivity changed: Offline");
                    }

                    if real_state.is_online() {
                        timeout = Box::pin(tokio::time::sleep(SYNTHETIC_OFFLINE_DURATION)).fuse();
                    }
                }
            }
        }

        tracing::trace!("Offline monitor exiting");
    });

    Ok(MonitorHandle {
        state,
        _notify_tx: notify_tx,
    })
}
