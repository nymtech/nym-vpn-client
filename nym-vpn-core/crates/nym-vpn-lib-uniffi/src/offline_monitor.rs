// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::sync::Arc;

use crate::{OFFLINE_MONITOR_HANDLE, error::VpnError};

#[cfg(target_os = "android")]
use crate::tunnel_provider::android::{
    AndroidConnectivityMonitor, ConnectivityObserverInvalidation, ConnectivityReceiver,
    ConnectivitySender,
};
use nym_offline_monitor::ConnectivityHandle;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_vpn_lib::tunnel_state_machine::{self, RouteHandler};

pub async fn init_offline_monitor(
    #[cfg(target_os = "android")] connectivity_monitor: Arc<dyn AndroidConnectivityMonitor>,
) -> Result<(), VpnError> {
    let mut guard = OFFLINE_MONITOR_HANDLE.lock().await;

    if guard.is_none() {
        let offline_monitor_handle = start_offline_monitor(
            #[cfg(target_os = "android")]
            connectivity_monitor,
        )
        .await?;
        *guard = Some(offline_monitor_handle);
        Ok(())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Offline monitor is already running.".to_owned(),
        })
    }
}

async fn start_offline_monitor(
    #[cfg(target_os = "android")] connectivity_monitor: Arc<dyn AndroidConnectivityMonitor>,
) -> Result<OfflineMonitorHandle, VpnError> {
    #[cfg(target_os = "linux")]
    let routing_params = tunnel_state_machine::RoutingParameters::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let route_handler = tunnel_state_machine::RouteHandler::new(
        #[cfg(target_os = "linux")]
        routing_params,
    )
    .await
    .map_err(tunnel_state_machine::Error::CreateRouteHandler)?;

    #[cfg(target_os = "android")]
    let connectivity_receiver = register_connectivity_observer(connectivity_monitor);

    let connectivity_handle = nym_offline_monitor::spawn_monitor(
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        route_handler.inner_handle(),
        #[cfg(target_os = "android")]
        connectivity_receiver,
        #[cfg(target_os = "linux")]
        Some(tunnel_state_machine::TUNNEL_FWMARK),
    )
    .await;

    Ok(OfflineMonitorHandle {
        connectivity_handle,
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        route_handler,
    })
}

#[cfg(target_os = "android")]
fn register_connectivity_observer(
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

pub struct OfflineMonitorHandle {
    connectivity_handle: ConnectivityHandle,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    route_handler: RouteHandler,
}

pub async fn get_connectivity_handle() -> Result<ConnectivityHandle, VpnError> {
    if let Some(guard) = &*OFFLINE_MONITOR_HANDLE.lock().await {
        Ok(guard.connectivity_handle.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Offline monitor is not running.".to_owned(),
        })
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn get_route_handler() -> Result<RouteHandler, VpnError> {
    if let Some(guard) = &*OFFLINE_MONITOR_HANDLE.lock().await {
        Ok(guard.route_handler.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Offline monitor is not running.".to_owned(),
        })
    }
}
