// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;

use nym_common::ErrorExt;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_routing::RouteManagerHandle;
use tokio::sync::watch;
use tokio_util::sync::{CancellationToken, DropGuard};

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[path = "apple/mod.rs"]
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

#[cfg(target_os = "android")]
pub use imp::NativeConnectivityAdapter;

/// Disables offline monitor
static FORCE_DISABLE_OFFLINE_MONITOR: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("NYM_DISABLE_OFFLINE_MONITOR")
        .map(|v| v != "0")
        .unwrap_or(false)
});

pub struct MonitorHandle {
    inner: Option<imp::MonitorHandle>,
    rx: watch::Receiver<Connectivity>,
    _shutdown_drop_guard: DropGuard,
}

impl MonitorHandle {
    fn new(
        inner: Option<imp::MonitorHandle>,
        rx: watch::Receiver<Connectivity>,
        shutdown_drop_guard: DropGuard,
    ) -> Self {
        Self {
            inner,
            rx,
            _shutdown_drop_guard: shutdown_drop_guard,
        }
    }

    /// Returns current connectivity status.
    pub async fn connectivity(&self) -> Connectivity {
        match self.inner.as_ref() {
            Some(monitor) => monitor.connectivity().await,
            None => Connectivity::PresumeOnline,
        }
    }

    /// Returns next connectivity status once changed.
    ///
    /// # Cancel safety
    ///
    /// This method is cancel safe as it uses the channel internally.
    pub async fn next(&mut self) -> Option<Connectivity> {
        if self.inner.is_some() {
            self.rx.changed().await.ok()?;
            Some(*self.rx.borrow_and_update())
        } else {
            None
        }
    }
}

/// Spawn offline monitor.
pub async fn spawn_monitor(
    #[cfg(not(any(target_os = "android", target_os = "ios")))] route_manager: RouteManagerHandle,
    #[cfg(target_os = "android")] connectivity_adapter: impl imp::NativeConnectivityAdapter + 'static,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> MonitorHandle {
    let (tx, rx) = watch::channel(Connectivity::PresumeOnline);
    let shutdown_token = CancellationToken::new();
    let child_token = shutdown_token.child_token();

    let monitor = if *FORCE_DISABLE_OFFLINE_MONITOR {
        tracing::info!("Offline monitor is disabled.");
        None
    } else {
        imp::spawn_monitor(
            tx,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_manager,
            #[cfg(target_os = "android")]
            connectivity_adapter,
            #[cfg(target_os = "linux")]
            fwmark,
            child_token,
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

    MonitorHandle::new(monitor, rx, shutdown_token.drop_guard())
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

    /// If the host does not have configured IPv6 routes, we have no way of
    /// reaching the internet so we consider ourselves offline.
    #[cfg(target_os = "android")]
    pub fn is_offline(&self) -> bool {
        matches!(self, Connectivity::Status { connected: false })
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
}
