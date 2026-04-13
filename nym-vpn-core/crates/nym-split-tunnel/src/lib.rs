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

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

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

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn effective_exclude_paths(
    paths: &HashSet<PathBuf>,
    socks5_proxy_path: Option<&Path>,
) -> HashSet<PathBuf> {
    let mut effective_paths = paths.clone();

    if let Some(socks5_proxy_path) = socks5_proxy_path {
        effective_paths.insert(socks5_proxy_path.to_path_buf());
    }

    effective_paths
}

#[cfg(test)]
mod tests {
    use super::effective_exclude_paths;
    use std::{collections::HashSet, path::PathBuf};

    #[test]
    fn effective_exclude_paths_returns_user_paths_when_no_proxy_path_is_set() {
        let user_path = PathBuf::from("C:/Program Files/App/app.exe");
        let user_paths = HashSet::from([user_path.clone()]);

        let effective_paths = effective_exclude_paths(&user_paths, None);

        assert_eq!(effective_paths, HashSet::from([user_path]));
    }

    #[test]
    fn effective_exclude_paths_adds_proxy_path() {
        let user_path = PathBuf::from("C:/Program Files/App/app.exe");
        let proxy_path = PathBuf::from("C:/Program Files/Nym/nym-socks5-proxy.exe");
        let user_paths = HashSet::from([user_path.clone()]);

        let effective_paths = effective_exclude_paths(&user_paths, Some(&proxy_path));

        assert_eq!(effective_paths, HashSet::from([user_path, proxy_path]));
    }

    #[test]
    fn effective_exclude_paths_deduplicates_proxy_path() {
        let proxy_path = PathBuf::from("C:/Program Files/Nym/nym-socks5-proxy.exe");
        let user_paths = HashSet::from([proxy_path.clone()]);

        let effective_paths = effective_exclude_paths(&user_paths, Some(&proxy_path));

        assert_eq!(effective_paths, HashSet::from([proxy_path]));
    }
}
