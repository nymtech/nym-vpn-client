// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::sync::Arc;

use crate::{OFFLINE_MONITOR_HANDLE, error::VpnError};

use nym_offline_monitor::ConnectivityHandle;
#[cfg(target_os = "android")]
use nym_vpn_lib::tunnel_provider::AndroidTunProvider;
#[cfg(target_os = "android")]
use nym_vpn_lib::tunnel_state_machine::AndroidConnectivityAdapter;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_vpn_lib::tunnel_state_machine::{self, RouteHandler};

pub(super) async fn init_offline_monitor(
    #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
) -> Result<(), VpnError> {
    let mut guard = OFFLINE_MONITOR_HANDLE.lock().await;

    if guard.is_none() {
        let offline_monitor_handle = start_offline_monitor(
            #[cfg(target_os = "android")]
            tun_provider,
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

pub(super) async fn start_offline_monitor(
    #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
) -> Result<OfflineMonitorHandle, VpnError> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let route_handler = tunnel_state_machine::RouteHandler::new()
        .await
        .map_err(tunnel_state_machine::Error::CreateRouteHandler)?;

    let connectivity_handle = nym_offline_monitor::spawn_monitor(
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        route_handler.inner_handle(),
        #[cfg(target_os = "android")]
        AndroidConnectivityAdapter::new(tun_provider),
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

pub(super) struct OfflineMonitorHandle {
    connectivity_handle: ConnectivityHandle,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    route_handler: RouteHandler,
}

pub(super) async fn get_connectivity_handle() -> Result<ConnectivityHandle, VpnError> {
    if let Some(guard) = &*OFFLINE_MONITOR_HANDLE.lock().await {
        Ok(guard.connectivity_handle.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Offline monitor is not running.".to_owned(),
        })
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(super) async fn get_route_handler() -> Result<RouteHandler, VpnError> {
    if let Some(guard) = &*OFFLINE_MONITOR_HANDLE.lock().await {
        Ok(guard.route_handler.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Offline monitor is not running.".to_owned(),
        })
    }
}
