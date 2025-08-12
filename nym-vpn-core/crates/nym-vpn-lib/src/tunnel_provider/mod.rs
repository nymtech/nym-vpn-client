// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

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
pub trait ConnectivityObserver: Send + Sync + std::fmt::Debug {
    fn on_network_change(&self, is_online: bool);
}

#[cfg(target_os = "android")]
pub trait AndroidTunProvider: Send + Sync + Debug {
    fn bypass(&self, socket: i32);
    fn configure_tunnel(&self, config: TunnelSettings) -> std::io::Result<RawFd>;

    fn add_connectivity_observer(&self, observer: Arc<dyn ConnectivityObserver>);
    fn remove_connectivity_observer(&self, observer: Arc<dyn ConnectivityObserver>);
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
