// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! NetworkManager constants and the small D-Bus proxy the plugin needs.

/// Well-known bus name NetworkManager expects this plugin to own (from the `.name` file).
pub const BUS_NAME: &str = "org.freedesktop.NetworkManager.nymvpn";
/// Object path every NetworkManager VPN plugin serves its interface at.
pub const OBJECT_PATH: &str = "/org/freedesktop/NetworkManager/VPN/Plugin";
/// The plugin interface name.
pub const INTERFACE: &str = "org.freedesktop.NetworkManager.VPN.Plugin";

/// `NMVpnServiceState` values.
pub mod service_state {
    pub const INIT: u32 = 1;
    pub const STARTING: u32 = 3;
    pub const STARTED: u32 = 4;
    pub const STOPPING: u32 = 5;
    pub const STOPPED: u32 = 6;
}

/// `NMVpnPluginFailure` values.
pub mod failure {
    pub const CONNECT_FAILED: u32 = 1;
    pub const BAD_IP_CONFIG: u32 = 2;
}

/// `NMState` threshold below which NetworkManager is asleep, disconnected or still connecting.
pub const NM_STATE_CONNECTED_LOCAL: u32 = 50;

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub trait NetworkManager {
    /// Overall NetworkManager state (`NMState`).
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;
}
