// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::routing::RoutingConfig;
use error::FFIError;
use ipnetwork::IpNetwork;
use oslog::OsLogger;
use std::fmt::Debug;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::os::fd::RawFd;
use talpid_types::net::wireguard::{
    PeerConfig as WgPeerConfig, PresharedKey, PrivateKey, PublicKey, TunnelConfig as WgTunnelConfig,
};

pub(crate) fn init_logs() {
    OsLogger::new("net.nymtech.vpn.agent")
        .level_filter(LevelFilter::Debug)
        .category_level_filter("hyper", log::LevelFilter::Warn)
        .category_level_filter("tokio_reactor", log::LevelFilter::Warn)
        .category_level_filter("reqwest", log::LevelFilter::Warn)
        .category_level_filter("mio", log::LevelFilter::Warn)
        .category_level_filter("want", log::LevelFilter::Warn)
        .category_level_filter("tungstenite", log::LevelFilter::Warn)
        .category_level_filter("tokio_tungstenite", log::LevelFilter::Warn)
        .category_level_filter("handlebars", log::LevelFilter::Warn)
        .category_level_filter("sled", log::LevelFilter::Warn)
        .init()
        .expect("Could not init logs");
    debug!("Logger initialized");
}

#[derive(uniffi::Record, Clone)]
pub struct TunnelConfig {
    pub private_key: PrivateKey,
    pub addresses: Vec<IpAddr>,
}

impl From<WgTunnelConfig> for TunnelConfig {
    fn from(value: WgTunnelConfig) -> Self {
        TunnelConfig {
            private_key: value.private_key,
            addresses: value.addresses,
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct PeerConfig {
    pub public_key: PublicKey,
    pub allowed_ips: Vec<IpNetwork>,
    pub endpoint: SocketAddr,
    pub psk: Option<PresharedKey>,
}

impl From<WgPeerConfig> for PeerConfig {
    fn from(value: WgPeerConfig) -> Self {
        PeerConfig {
            public_key: value.public_key,
            allowed_ips: value.allowed_ips,
            endpoint: value.endpoint,
            psk: value.psk,
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct WgConfig {
    pub tunnel: TunnelConfig,
    pub peers: Vec<PeerConfig>,
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Option<Ipv6Addr>,
    pub mtu: u16,
}

impl From<talpid_wireguard::config::Config> for WgConfig {
    fn from(value: talpid_wireguard::config::Config) -> Self {
        WgConfig {
            tunnel: value.tunnel.into(),
            peers: value.peers.into_iter().map(Into::into).collect(),
            ipv4_gateway: value.ipv4_gateway,
            ipv6_gateway: value.ipv6_gateway,
            mtu: value.mtu,
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct NymConfig {
    pub ipv4_addr: Ipv4Addr,
    pub ipv6_addr: Ipv6Addr,
    pub mtu: u16,
    pub entry_mixnet_gateway_ip: Option<IpAddr>,
}

impl From<RoutingConfig> for NymConfig {
    fn from(value: RoutingConfig) -> Self {
        NymConfig {
            ipv4_addr: value.tun_ips().ipv4,
            ipv6_addr: value.tun_ips().ipv6,
            mtu: value.mtu(),
            entry_mixnet_gateway_ip: None,
        }
    }
}

#[cfg(target_os = "ios")]
#[derive(uniffi::Record)]
pub struct Ipv4AddrRange {
    pub address: Ipv4Addr,
    pub netmask: Ipv4Addr,
}

#[cfg(target_os = "ios")]
#[derive(uniffi::Record)]
pub struct Ipv6AddrRange {
    pub address: Ipv6Addr,
    pub prefix_length: u16,
}

#[cfg(target_os = "ios")]
#[derive(uniffi::Record)]
pub struct Ipv4Route {
    pub destination: Ipv4Addr,
    pub subnet_mask: Ipv4Addr,
    pub gateway: Option<Ipv4Addr>,
}

#[cfg(target_os = "ios")]
#[derive(uniffi::Record)]
pub struct Ipv6Route {
    pub destination: Ipv6Addr,
    pub network_prefix_length: u16, // clamp to /120
    pub gateway: Option<Ipv6Addr>,
}

#[cfg(target_os = "ios")]
#[derive(uniffi::Record)]
pub struct Ipv4Settings {
    /// IPv4 addresses that will be set on tunnel interface.
    pub addresses: Vec<Ipv4AddrRange>,

    /// Traffic matching these routes will be routed over the tun interface.
    pub included_routes: Vec<Ipv4Route>,

    /// Traffic matching these routes will be routed over the primary physical interface.
    pub excluded_routes: Vec<Ipv4Route>,
}

#[cfg(target_os = "ios")]
#[derive(uniffi::Record)]
pub struct Ipv6Settings {
    /// IPv4 addresses that will be set on tunnel interface.
    pub addresses: Vec<Ipv6AddrRange>,

    /// Traffic matching these routes will be routed over the tun interface.
    pub included_routes: Vec<Ipv6Route>,

    /// Traffic matching these routes will be routed over the primary physical interface.
    pub excluded_routes: Vec<Ipv6Route>,
}

#[cfg(target_os = "ios")]
#[derive(uniffi::Record)]
pub struct TunnelNetworkSettings {
    /// Tunnel remote address, which is mostly of decorative value.
    pub tunnel_remote_addr: String,

    /// IPv4 interface settings.
    pub ipv4_settings: Ipv4Settings,

    /// IPv6 interface settings.
    pub ipv6_settings: Ipv6Settings,

    /// Tunnel device MTU.
    pub mtu: u16,
}

#[uniffi::export(with_foreign)]
pub trait OSTunProvider: Send + Sync + Debug {
    fn configure_wg(&self, config: WgConfig) -> Result<(), FFIError>;
    fn configure_nym(&self, config: NymConfig) -> Result<RawFd, FFIError>;

    #[cfg(target_os = "ios")]
    fn set_tunnel_network_settings(
        &self,
        tunnel_settings: TunnelNetworkSettings,
    ) -> Result<(), FFIError>;
}
