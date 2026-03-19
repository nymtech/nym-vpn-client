// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(target_os = "linux")]
pub use imp::*;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod imp;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use imp::*;

#[cfg(target_os = "windows")]
pub use imp::{install_driver_service, uninstall_driver_service};

use std::net::{Ipv4Addr, Ipv6Addr};

/// VPN tunnel interface configuration used by split tunneling.
#[derive(Debug, Clone)]
pub struct VpnInterface {
    /// VPN tunnel interface name
    pub name: String,
    /// VPN tunnel IPv4 address
    pub v4_address: Option<Ipv4Addr>,
    /// VPN tunnel IPv6 address
    pub v6_address: Option<Ipv6Addr>,
}

/// Type describing what caused split tunneling to fail.
#[derive(Debug, Copy, Clone)]
pub enum SplitTunnelErrorCause {
    /// Device is offline, split tunneling cannot be used.
    #[cfg(target_os = "macos")]
    IsOffline,

    #[cfg(target_os = "macos")]
    /// Full disk permissions are required to use split tunneling.
    NeedFullDiskPermissions,

    /// Other error
    Other,
}
