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

/// `NMActiveConnectionState::DEACTIVATING`.
///
/// A VPN connection only passes through this state when it is deactivated on request (the user
/// disconnecting it or deleting its profile). Losing the parent device, sleep and plugin failures
/// take it straight to DEACTIVATED.
pub const ACTIVE_CONNECTION_STATE_DEACTIVATING: u32 = 3;

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub trait NetworkManager {
    fn activate_connection(
        &self,
        connection: &zbus::zvariant::ObjectPath<'_>,
        device: &zbus::zvariant::ObjectPath<'_>,
        specific_object: &zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
pub trait Settings {
    fn get_connection_by_uuid(&self, uuid: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait ActiveConnection {
    #[zbus(property)]
    fn uuid(&self) -> zbus::Result<String>;

    #[zbus(signal)]
    fn state_changed(&self, state: u32, reason: u32) -> zbus::Result<()>;
}
