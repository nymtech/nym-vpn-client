// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod manager;
mod process;

pub use manager::Socks5ProxyManager;
pub use process::find_proxy_binary;

use std::net::IpAddr;

#[cfg(target_os = "windows")]
use nym_routing::get_best_default_route;

#[cfg(target_os = "windows")]
use nym_windows::net::{AddressFamily, get_ip_address_for_interface};

#[cfg(windows)]
pub(crate) fn get_default_addr_sync() -> Option<IpAddr> {
    match get_best_default_route(AddressFamily::Ipv4) {
        Ok(Some(route)) => match get_ip_address_for_interface(AddressFamily::Ipv4, route.iface) {
            Ok(Some(ip)) => {
                tracing::debug!("Resolved default address: {ip}");
                return Some(ip);
            }
            Ok(None) => {
                tracing::warn!("Default interface has no IPv4 address");
            }
            Err(err) => {
                tracing::warn!("Failed to get default IP address: {err}");
            }
        },
        Ok(None) => {
            tracing::warn!("No physical default IPv4 route found");
        }
        Err(err) => {
            tracing::warn!("get_best_default_route failed: {err}");
        }
    }
    None
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) async fn get_default_addr(
    route_manager: &nym_routing::RouteManagerHandle,
) -> Option<IpAddr> {
    #[cfg(target_os = "windows")]
    {
        let _ = route_manager; // Avoid unused variable warning
        get_default_addr_sync()
    }
    #[cfg(target_os = "macos")]
    {
        route_manager
            .get_default_routes()
            .await
            .ok()
            .and_then(|(v4, _)| v4.map(|route| route.ip))
    }
    #[cfg(target_os = "linux")]
    {
        let _ = route_manager; // Avoid unused variable warning
        None
    }
}
