// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

#[cfg(target_os = "linux")]
use nym_routing::RouteManagerHandle;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod imp;

#[cfg(any(target_os = "android", target_os = "ios"))]
#[path = "android.rs"]
mod imp;

pub use self::imp::Error;
#[cfg(windows)]
pub use imp::flush_resolver_cache;

/// DNS configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsConfig {
    /// DNS server addresses
    pub addresses: Vec<IpAddr>,

    /// Port to use
    pub port: u16,
}

/// Sets and monitors system DNS settings. Makes sure the desired DNS servers are being used.
pub struct DnsMonitor {
    inner: imp::DnsMonitor,
}

impl DnsMonitor {
    /// Returns a new `DnsMonitor` that can set and monitor the system DNS.
    pub fn new(
        #[cfg(target_os = "linux")] route_manager: RouteManagerHandle,
    ) -> Result<Self, Error> {
        Ok(DnsMonitor {
            inner: imp::DnsMonitor::new(
                #[cfg(target_os = "linux")]
                route_manager,
            )?,
        })
    }

    /// Set DNS to the given servers. And start monitoring the system for changes.
    pub async fn set(&mut self, interface: &str, config: DnsConfig) -> Result<(), Error> {
        tracing::info!("Setting DNS servers on interface '{interface}': {config:?}");
        self.inner.set(interface, config).await
    }

    /// Reset system DNS settings to what it was before being set by this instance.
    /// This succeeds if the interface does not exist.
    pub async fn reset(&mut self) -> Result<(), Error> {
        tracing::info!("Resetting DNS");
        self.inner.reset().await
    }

    /// Reset DNS settings to what they were before being set by this instance.
    /// If the settings only affect a specific interface, this can be a no-op,
    /// as the interface will be destroyed.
    pub async fn reset_before_interface_removal(&mut self) -> Result<(), Error> {
        tracing::info!("Resetting DNS");
        self.inner.reset_before_interface_removal().await
    }
}

trait DnsMonitorT: Sized {
    type Error: std::error::Error;

    fn new(
        #[cfg(target_os = "linux")] route_manager: RouteManagerHandle,
    ) -> Result<Self, Self::Error>;

    async fn set(&mut self, interface: &str, servers: DnsConfig) -> Result<(), Self::Error>;

    async fn reset(&mut self) -> Result<(), Self::Error>;

    async fn reset_before_interface_removal(&mut self) -> Result<(), Self::Error> {
        self.reset().await
    }
}
