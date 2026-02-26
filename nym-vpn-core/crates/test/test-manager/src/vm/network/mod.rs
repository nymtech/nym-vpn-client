// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// #[cfg(target_os = "linux")]
pub mod linux;
use std::net::Ipv4Addr;

#[cfg(target_os = "linux")]
pub use linux as platform;

#[cfg(target_os = "macos")]
pub mod macos;

/// Get the name of the bridge interface between the test-manager and the test-runner.
pub fn bridge(
    #[cfg(target_os = "macos")] bridge_ip: &Ipv4Addr,
) -> anyhow::Result<(String, Ipv4Addr)> {
    #[cfg(target_os = "macos")]
    {
        crate::vm::network::macos::find_vm_bridge(bridge_ip)
    }
    #[cfg(target_os = "linux")]
    {
        Ok((platform::BRIDGE_NAME.to_owned(), platform::NON_TUN_GATEWAY))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("bridge() is not implemented on this platform")
    }
}
