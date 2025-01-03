// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;

use futures::channel::mpsc::UnboundedSender;
use nym_common::ErrorExt;
#[cfg(not(target_os = "android"))]
use nym_routing::RouteManagerHandle;

#[cfg(target_os = "android")]
use crate::connectivity_listener::ConnectivityListener;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

/// Disables offline monitor
static FORCE_DISABLE_OFFLINE_MONITOR: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("NYM_DISABLE_OFFLINE_MONITOR")
        .map(|v| v != "0")
        .unwrap_or(false)
});

pub struct MonitorHandle(Option<imp::MonitorHandle>);

impl MonitorHandle {
    pub async fn connectivity(&self) -> Connectivity {
        match self.0.as_ref() {
            Some(monitor) => monitor.connectivity().await,
            None => Connectivity::PresumeOnline,
        }
    }
}

pub async fn spawn_monitor(
    sender: UnboundedSender<Connectivity>,
    #[cfg(not(target_os = "android"))] route_manager: RouteManagerHandle,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
    #[cfg(target_os = "android")] connectivity_listener: ConnectivityListener,
) -> MonitorHandle {
    let monitor = if *FORCE_DISABLE_OFFLINE_MONITOR {
        None
    } else {
        imp::spawn_monitor(
            sender,
            #[cfg(not(target_os = "android"))]
            route_manager,
            #[cfg(target_os = "linux")]
            fwmark,
            #[cfg(target_os = "android")]
            connectivity_listener,
        )
        .await
        .inspect_err(|error| {
            tracing::warn!(
                "{}",
                error.display_chain_with_msg("Failed to spawn offline monitor")
            );
        })
        .ok()
    };

    MonitorHandle(monitor)
}

/// Details about the hosts's connectivity.
///
/// Information about the host's connectivity, such as the preesence of
/// configured IPv4 and/or IPv6.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Connectivity {
    #[cfg(not(target_os = "android"))]
    Status {
        /// Whether IPv4 connectivity seems to be available on the host.
        ipv4: bool,
        /// Whether IPv6 connectivity seems to be available on the host.
        ipv6: bool,
    },
    #[cfg(any(target_os = "ios", target_os = "android"))]
    Status {
        /// Whether _any_ connectivity seems to be available on the host.
        connected: bool,
    },
    /// On/offline status could not be verified, but we have no particular
    /// reason to believe that the host is offline.
    PresumeOnline,
}

impl Connectivity {
    /// Inverse of [`Connectivity::is_offline`].
    pub fn is_online(&self) -> bool {
        !self.is_offline()
    }

    /// If no IP4 nor IPv6 routes exist, we have no way of reaching the internet
    /// so we consider ourselves offline.
    #[cfg(not(target_os = "android"))]
    pub fn is_offline(&self) -> bool {
        matches!(
            self,
            Connectivity::Status {
                ipv4: false,
                ipv6: false
            }
        )
    }

    /// Whether IPv6 connectivity seems to be available on the host.
    ///
    /// If IPv6 status is unknown, `false` is returned.
    #[cfg(not(target_os = "android"))]
    pub fn has_ipv6(&self) -> bool {
        matches!(self, Connectivity::Status { ipv6: true, .. })
    }

    /// Whether IPv6 connectivity seems to be available on the host.
    ///
    /// If IPv6 status is unknown, `false` is returned.
    #[cfg(target_os = "android")]
    pub fn has_ipv6(&self) -> bool {
        self.is_online()
    }

    /// If the host does not have configured IPv6 routes, we have no way of
    /// reaching the internet so we consider ourselves offline.
    #[cfg(target_os = "android")]
    pub fn is_offline(&self) -> bool {
        matches!(self, Connectivity::Status { connected: false })
    }
}
