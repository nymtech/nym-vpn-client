// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::routing::RoutingConfig;
use ipnetwork::IpNetwork;
use oslog::OsLogger;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use talpid_types::net::wireguard::{
    PeerConfig as WgPeerConfig, PresharedKey, PrivateKey, PublicKey, TunnelConfig as WgTunnelConfig,
};
use std::fmt::Debug;
use std::os::fd::RawFd;

pub fn init_logs(level: String) {
    OsLogger::new("net.nymtech.vpn.agent")
        .level_filter(log::LevelFilter::from_str(level.as_str()).expect("Invalid log level"))
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

#[uniffi::export(with_foreign)]
pub trait OSTunProvider: Send + Sync + Debug {
    fn configure_wg(&self, config: WgConfig) -> Result<(), crate::platform::error::FFIError>;
    fn configure_nym(&self, config: NymConfig) -> Result<RawFd, crate::platform::error::FFIError>;
}
