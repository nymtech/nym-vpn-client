// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "ios")]
pub mod ios;

use std::net::IpAddr;

use ipnetwork::IpNetwork;

#[cfg(target_os = "ios")]
#[async_trait::async_trait]
pub trait OSTunProvider: Send + Sync + std::fmt::Debug {
    /// Set network settings including tun, dns, ip.
    async fn set_tunnel_network_settings(
        &self,
        tunnel_settings: TunnelSettings,
    ) -> std::io::Result<()>;
}

#[cfg(target_os = "android")]
pub trait AndroidTunProvider: Send + Sync + std::fmt::Debug {
    fn bypass(&self, socket: i32);
    fn configure_tunnel(&self, config: TunnelSettings) -> std::io::Result<std::os::fd::RawFd>;
    /// Resolve the UID owning a connection. protocol: 6 = TCP, 17 = UDP;
    /// source/destination: "ip:port" ("[ip]:port" for IPv6). Returns -1 when
    /// the owner cannot be determined.
    fn get_connection_owner_uid(&self, protocol: i32, source: String, destination: String) -> i32;
}

/// Per-connection app-bypass ("steering") configuration for Android lockdown
/// mode.
///
/// `None`/absent means steering is off and classic per-app exclusion
/// (`VpnService.Builder.addDisallowedApplication`) is in effect. When present,
/// the real TUN device is handed to the libwg steering engine, which routes
/// flows owned by `excluded_uids` directly to the network (over `protect()`-ed
/// sockets) instead of into the tunnel.
#[cfg(target_os = "android")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppBypassConfig {
    /// UIDs of the apps whose traffic must bypass the tunnel.
    pub excluded_uids: Vec<u32>,

    /// DNS servers of the underlying (non-VPN) network, used to answer DNS
    /// queries of the bypassed apps.
    pub underlying_dns: Vec<IpAddr>,

    /// Forward flows destined for local-network ranges directly (LAN bypass).
    /// Under the kill switch the usual route-based LAN exemption is blocked, so
    /// the steering engine must bypass local-network destinations instead.
    pub bypass_lan: bool,

    /// The underlying network's real local subnet(s) as CIDR strings. Used
    /// (only when `bypass_lan` is set) as the exact ranges to bypass, so the
    /// tunnel's own in-tunnel RFC1918 addresses are not mistaken for LAN.
    pub lan_prefixes: Vec<String>,
}

#[derive(Debug)]
pub struct TunnelSettings {
    /// Tunnel interface addresses.
    pub interface_addresses: Vec<IpNetwork>,

    /// DNS servers to set on tunnel interface.
    pub dns_servers: Vec<IpAddr>,

    /// Tunnel remote addresses that will be excluded from being routed over the tunnel
    /// to prevent the network loop.
    pub remote_addresses: Vec<IpAddr>,

    /// Tunnel device MTU.
    pub mtu: u16,
}
