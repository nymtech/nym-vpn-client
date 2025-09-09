// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod storage;

mod bandwidth_controller;
mod error;
mod mixnet;
#[cfg(any(target_os = "ios", target_os = "android"))]
pub mod tunnel_provider;
pub mod tunnel_state_machine;
mod wg_config;

use std::{net::IpAddr, sync::LazyLock};

use hickory_resolver::config::NameServerConfigGroup;
use itertools::Itertools;

// Re-export some our nym dependencies
pub use nym_authenticator_client::Error as AuthenticatorClientError;
pub use nym_config;
pub use nym_connection_monitor as connection_monitor;
pub use nym_gateway_directory as gateway_directory;
pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::{
    UserAgent,
    mixnet::{NodeIdentity, Recipient, StoragePaths},
};
pub use nym_task::{
    StatusReceiver,
    event::{SentStatus, TaskStatus},
};
pub use nym_wg_gateway_client as wg_gateway_client;

pub use crate::{
    error::GatewayDirectoryError,
    mixnet::{MixnetError, VpnTopologyProvider},
};

/// Default DNS servers.
static DEFAULT_DNS_SERVERS_CONFIG: LazyLock<NameServerConfigGroup> = LazyLock::new(|| {
    let mut name_servers = NameServerConfigGroup::quad9_tls();
    name_servers.merge(NameServerConfigGroup::quad9_https());
    name_servers.merge(NameServerConfigGroup::cloudflare_tls());
    name_servers.merge(NameServerConfigGroup::cloudflare_https());
    name_servers
});

/// Default DNS server IP addresses.
pub(crate) static DEFAULT_DNS_SERVERS: LazyLock<Vec<IpAddr>> = LazyLock::new(|| {
    DEFAULT_DNS_SERVERS_CONFIG
        .iter()
        .map(|ns| ns.socket_addr.ip())
        .unique()
        .collect()
});

/// Routing table id used for routing all traffic through the tunnel.
#[cfg(target_os = "linux")]
pub const TUNNEL_TABLE_ID: u32 = 0x14d;

/// Firewall mark used for marking traffic that should bypass the tunnel.
#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x14d;

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
