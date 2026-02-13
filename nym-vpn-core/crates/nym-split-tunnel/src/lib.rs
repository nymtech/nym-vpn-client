// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod imp;

pub use imp::*;

/// Type describing what caused split tunneling to fail.
#[derive(Debug, Copy, Clone)]
pub enum SplitTunnelErrorCause {
    /// Device is offline, split tunneling cannot be used.
    IsOffline,

    #[cfg(target_os = "macos")]
    /// Full disk permissions are required to use split tunneling.
    NeedFullDiskPermissions,

    /// Other error
    Other,
}
