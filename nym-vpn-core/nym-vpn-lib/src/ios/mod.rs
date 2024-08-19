// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

//! WireGuard tunnel creation and management on Android and iOS
//! todo: the location of this module will be changed.

mod dns64;
mod gateway;
pub mod tun;
pub mod tunnel_settings;
pub mod two_hop_tunnel;

use std::net::SocketAddr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to locate tun fd")]
    CannotLocateTunFd,

    #[error("Failed to obtain tun interface name")]
    ObtainTunName,

    #[error("Tunnel failure")]
    Tunnel(nym_wg_go::Error),

    #[error("Failed to resolve {} (error code: {})", addr, code)]
    DnsLookup { code: i32, addr: SocketAddr },

    #[error("Failed to parse addrinfo")]
    ParseAddrInfo(std::io::Error),

    #[error("DNS lookup has seemingly succeeded without any results")]
    EmptyDnsLookupResult,

    #[error("Failed to convert port to C-string")]
    ConvertPortToCstr,

    #[error("Failed to convert ip to C-string")]
    ConvertIpToCstr,

    #[error("Invalid WireGuard key")]
    InvalidKey,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait OSTunProvider: Send + Sync + std::fmt::Debug {
    async fn set_tunnel_network_settings(
        &self,
        tunnel_settings: tunnel_settings::TunnelNetworkSettings,
    ) -> std::result::Result<(), crate::platform::error::FFIError>;
}
