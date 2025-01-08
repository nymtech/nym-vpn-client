// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(windows)]
mod window;

use std::sync::LazyLock;

use nym_common::ErrorExt;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_routing::RouteManagerHandle;
use tokio::sync::mpsc;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "ios")]
#[path = "ios.rs"]
mod imp;

#[cfg(windows)]
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
    sender: mpsc::UnboundedSender<Connectivity>,
    #[cfg(not(any(target_os = "android", target_os = "ios")))] route_manager: RouteManagerHandle,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> MonitorHandle {
    let monitor = if *FORCE_DISABLE_OFFLINE_MONITOR {
        None
    } else {
        imp::spawn_monitor(
            sender,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_manager,
            #[cfg(target_os = "linux")]
            fwmark,
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
    #[cfg(target_os = "android")]
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
