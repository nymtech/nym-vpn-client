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
