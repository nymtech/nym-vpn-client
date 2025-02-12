// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

pub mod storage;
pub mod util;

mod bandwidth_controller;
mod error;
mod mixnet;
mod platform;
#[cfg(any(target_os = "ios", target_os = "android"))]
pub mod tunnel_provider;
pub mod tunnel_state_machine;
mod wg_config;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Re-export some our nym dependencies
pub use nym_authenticator_client::Error as AuthenticatorClientError;
pub use nym_config;
pub use nym_connection_monitor as connection_monitor;
pub use nym_gateway_directory as gateway_directory;
pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::{
    mixnet::{NodeIdentity, Recipient, StoragePaths},
    UserAgent,
};
pub use nym_task::{
    event::{SentStatus, TaskStatus},
    StatusReceiver,
};
pub use nym_wg_gateway_client as wg_gateway_client;

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use crate::platform::swift;
pub use crate::{
    error::{Error, GatewayDirectoryError},
    mixnet::MixnetError,
};

pub const DEFAULT_DNS_SERVERS: [IpAddr; 4] = [
    IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)),
    IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111)),
    IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1001)),
];

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct MixnetClientConfig {
    /// Disable Poission process rate limiting of outbound traffic.
    pub disable_poisson_rate: bool,

    /// Disable constant rate background loop cover traffic
    pub disable_background_cover_traffic: bool,

    /// The minimum performance of mixnodes to use.
    pub min_mixnode_performance: Option<u8>,

    /// The minimum performance of gateways to use.
    pub min_gateway_performance: Option<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct MixnetConnectionInfo {
    pub nym_address: Recipient,
    pub entry_gateway: NodeIdentity,
}

#[derive(Debug, Clone, Copy)]
pub struct MixnetExitConnectionInfo {
    pub exit_gateway: NodeIdentity,
    pub exit_ipr: Recipient,
    pub ips: IpPair,
}
