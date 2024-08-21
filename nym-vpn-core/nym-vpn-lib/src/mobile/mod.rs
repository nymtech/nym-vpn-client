// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

//! WireGuard tunnel creation and management on Android and iOS
//! todo: the location of this module will be changed.

mod default_path_observer;
mod dns64;
mod gateway;
pub mod tun;
pub mod tunnel_settings;
pub mod two_hop_config;
pub mod two_hop_tunnel;
mod wg_config;

use std::net::SocketAddr;
use std::sync::Arc;

use crate::platform::error::FFIError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to locate tun fd")]
    CannotLocateTunFd,

    #[error("Failed to obtain tun interface name")]
    ObtainTunName,

    #[error("Tunnel failure")]
    Tunnel(#[from] nym_wg_go::Error),

    #[error("Failed to resolve {} (error code: {})", addr, code)]
    DnsLookup { code: i32, addr: SocketAddr },

    #[error("Failed to parse addrinfo")]
    ParseAddrInfo(#[source] std::io::Error),

    #[error("DNS lookup has seemingly succeeded without any results")]
    EmptyDnsLookupResult,

    #[error("Failed to convert port to C-string")]
    ConvertPortToCstr,

    #[error("Failed to convert ip to C-string")]
    ConvertIpToCstr,

    #[error("Invalid WireGuard key")]
    InvalidKey,

    #[error("Failed to set network settings")]
    SetNetworkSettings(#[source] FFIError),

    #[error("Failed to set default path observer")]
    SetDefaultPathObserver(#[source] FFIError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(uniffi::Enum, Debug)]
pub enum OSPathStatus {
    /// The path cannot be evaluated.
    Invalid,

    /// The path is ready to be used for network connections.
    Satisfied,

    /// The path for network connections is not available, either due to lack of network
    /// connectivity or being prohibited by system policy.
    Unsatisfied,

    /// The path is not currently satisfied, but may become satisfied upon a connection attempt.
    /// This can be due to a service, such as a VPN or a cellular data connection not being activated.
    Satisfiable,

    /// Unknown path status was received.
    /// The raw variant code is contained in associated value.
    Unknown(i64),
}

/// Represents a default network route used by the system.
#[derive(uniffi::Record, Debug)]
pub struct OSDefaultPath {
    /// Indicates whether the process is able to make connection through the given path.
    pub status: OSPathStatus,

    /// Set to true for interfaces that are considered expensive, such as when using cellular data plan.
    pub is_expensive: bool,

    /// Set to true when using a constrained interface, such as when using low-data mode.
    pub is_constrained: bool,
}

/// Types observing network changes.
#[uniffi::export(with_foreign)]
pub trait OSDefaultPathObserver: Send + Sync + std::fmt::Debug {
    fn on_default_path_change(&self, new_path: OSDefaultPath);
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait OSTunProvider: Send + Sync + std::fmt::Debug {
    /// Set network settings including tun, dns, ip.
    async fn set_tunnel_network_settings(
        &self,
        tunnel_settings: tunnel_settings::TunnelNetworkSettings,
    ) -> std::result::Result<(), FFIError>;

    /// Set or unset the default path observer.
    fn set_default_path_observer(
        &self,
        observer: Option<Arc<dyn OSDefaultPathObserver>>,
    ) -> std::result::Result<(), FFIError>;
}
