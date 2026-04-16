// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use tokio::sync::watch;

pub async fn start_monitor() -> watch::Receiver<Option<IpAddr>> {
    let initial = get_current_default_addr().await;
    let (tx, rx) = watch::channel(initial);

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        tokio::spawn(monitor_task(tx));
    }

    // On Linux (and other platforms) there is nothing to monitor.
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = tx; // keep the sender alive – the value will never change
    }

    rx
}

#[cfg(target_os = "windows")]
async fn get_current_default_addr() -> Option<IpAddr> {
    use nym_routing::get_best_default_route;
    use nym_windows::net::{AddressFamily, get_ip_address_for_interface};

    match get_best_default_route(AddressFamily::Ipv4) {
        Ok(Some(route)) => match get_ip_address_for_interface(AddressFamily::Ipv4, route.iface) {
            Ok(Some(ip)) => {
                tracing::debug!("Initial default address: {ip}");
                Some(ip)
            }
            Ok(None) => {
                tracing::warn!("Default interface has no IPv4 address");
                None
            }
            Err(err) => {
                tracing::warn!("Failed to get default IP address: {err}");
                None
            }
        },
        Ok(None) => {
            tracing::warn!("No physical default IPv4 route found");
            None
        }
        Err(err) => {
            tracing::warn!("get_best_default_route failed: {err}");
            None
        }
    }
}

#[cfg(target_os = "windows")]
async fn monitor_task(tx: watch::Sender<Option<IpAddr>>) {
    use nym_routing::RouteManagerHandle;

    let route_manager = match RouteManagerHandle::spawn().await {
        Ok(rm) => rm,
        Err(err) => {
            tracing::warn!("Failed to start route manager for default-addr monitor: {err}");
            return;
        }
    };

    let callback: nym_routing::Callback = Box::new(move |event, _family| {
        use nym_routing::EventType;
        match event {
            EventType::Updated(_) | EventType::UpdatedDetails(_) => {
                let ip = {
                    use nym_routing::get_best_default_route;
                    use nym_windows::net::{AddressFamily, get_ip_address_for_interface};
                    match get_best_default_route(AddressFamily::Ipv4) {
                        Ok(Some(route)) => {
                            get_ip_address_for_interface(AddressFamily::Ipv4, route.iface)
                                .ok()
                                .flatten()
                        }
                        _ => None,
                    }
                };
                tracing::debug!("Default interface changed; new IP: {ip:?}");
                let _ = tx.send(ip);
            }
            EventType::Removed => {
                tracing::debug!("Default route removed");
                let _ = tx.send(None);
            }
        }
    });

    match route_manager
        .add_default_route_change_callback(callback)
        .await
    {
        Ok(_handle) => {
            // Keep the handle alive forever (it is dropped when the task exits).
            // The task runs until the tokio runtime shuts down.
            std::future::pending::<()>().await;
        }
        Err(err) => {
            tracing::warn!("Failed to register default route change callback: {err}");
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
async fn get_current_default_addr() -> Option<IpAddr> {
    use nym_routing::RouteManagerHandle;

    let route_manager = match RouteManagerHandle::spawn().await {
        Ok(rm) => rm,
        Err(err) => {
            tracing::warn!("Failed to start route manager to get initial default addr: {err}");
            return None;
        }
    };

    let result = route_manager
        .get_default_routes()
        .await
        .ok()
        .and_then(|(v4, _)| v4.map(|r| r.ip));

    route_manager.stop().await;
    result
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
async fn monitor_task(tx: watch::Sender<Option<IpAddr>>) {
    use nym_routing::{DefaultRouteEvent, RouteManagerHandle};

    let route_manager = match RouteManagerHandle::spawn().await {
        Ok(rm) => rm,
        Err(err) => {
            tracing::warn!("Failed to start route manager for default-addr monitor: {err}");
            return;
        }
    };

    let mut listener = match route_manager.default_route_listener().await {
        Ok(rx) => rx,
        Err(err) => {
            tracing::warn!("Failed to subscribe to default route changes: {err}");
            route_manager.stop().await;
            return;
        }
    };

    while let Some(event) = listener.recv().await {
        match event {
            DefaultRouteEvent::AddedOrChangedV4 => match route_manager.get_default_routes().await {
                Ok((Some(route), _)) => {
                    tracing::debug!("Default interface changed to {ip}", ip = route.ip);
                    let _ = tx.send(Some(route.ip));
                }
                Ok((None, _)) => {
                    tracing::debug!("Default IPv4 route changed but is now absent");
                    let _ = tx.send(None);
                }
                Err(err) => {
                    tracing::warn!("Failed to query default route after change event: {err}");
                }
            },
            DefaultRouteEvent::RemovedV4 => {
                tracing::debug!("Default IPv4 route removed");
                let _ = tx.send(None);
            }
            DefaultRouteEvent::AddedOrChangedV6 | DefaultRouteEvent::RemovedV6 => {}
        }
    }

    route_manager.stop().await;
}
