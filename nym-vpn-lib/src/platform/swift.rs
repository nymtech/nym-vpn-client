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
use talpid_types::net::wireguard::{PeerConfig, PresharedKey, PrivateKey, PublicKey, TunnelConfig};

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
pub struct UniffiTunnelConfig {
    pub private_key: PrivateKey,
    pub addresses: Vec<IpAddr>,
}

impl From<TunnelConfig> for UniffiTunnelConfig {
    fn from(value: TunnelConfig) -> Self {
        UniffiTunnelConfig {
            private_key: value.private_key,
            addresses: value.addresses,
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct UniffiPeerConfig {
    pub public_key: PublicKey,
    pub allowed_ips: Vec<IpNetwork>,
    pub endpoint: SocketAddr,
    pub psk: Option<PresharedKey>,
}

impl From<PeerConfig> for UniffiPeerConfig {
    fn from(value: PeerConfig) -> Self {
        UniffiPeerConfig {
            public_key: value.public_key,
            allowed_ips: value.allowed_ips,
            endpoint: value.endpoint,
            psk: value.psk,
        }
    }
}

#[derive(uniffi::Record, Clone)]
pub struct WgConfig {
    pub tunnel: UniffiTunnelConfig,
    pub peers: Vec<UniffiPeerConfig>,
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
        let entry_mixnet_gateway_ip = if value.enable_wireguard() {
            Some(value.entry_mixnet_gateway_ip())
        } else {
            None
        };
        NymConfig {
            ipv4_addr: value.tun_ips().ipv4,
            ipv6_addr: value.tun_ips().ipv6,
            mtu: value.mtu(),
            entry_mixnet_gateway_ip,
        }
    }
}

#[uniffi::export(with_foreign)]
pub trait OSTunProvider: Send + Sync + Debug {
    fn configure_wg(&self, config: WgConfig) -> Result<(), FFIError>;
    fn configure_nym(&self, config: NymConfig) -> Result<RawFd, FFIError>;
}
