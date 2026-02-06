// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod storage;

mod bandwidth_controller;
pub mod cache_refresh;
pub mod config;
mod error;
pub mod logging;
mod mixnet;
pub mod privy;
pub mod sentry;
pub mod service;
#[cfg(any(target_os = "ios", target_os = "android"))]
pub mod tunnel_provider;
pub mod tunnel_state_machine;
mod wg_config;

use std::{net::IpAddr, sync::LazyLock};

use hickory_resolver::config::NameServerConfigGroup;
use itertools::Itertools;

// Re-export some our nym dependencies
pub use nym_config;
pub use nym_gateway_directory as gateway_directory;
pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::{
    UserAgent,
    mixnet::{NodeIdentity, Recipient, StoragePaths},
};

pub use crate::{
    error::GatewayDirectoryError,
    mixnet::{
        DEFAULT_MIN_GATEWAY_PERFORMANCE, DEFAULT_MIN_MIXNODE_PERFORMANCE, MixnetError,
        VpnTopologyProvider, VpnTopologyService, VpnTopologyServiceError, VpnTopologyServiceHandle,
    },
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
pub static DEFAULT_DNS_SERVERS: LazyLock<Vec<IpAddr>> = LazyLock::new(|| {
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

pub fn log_software_and_os_version() {
    let build_info = nym_bin_common::bin_info_local_vergen!();
    tracing::info!(
        "{} {} ({})",
        build_info.binary_name,
        build_info.build_version,
        build_info.commit_sha
    );

    let os = nym_platform_metadata::SysInfo::new();
    tracing::info!("OS information: {}", os);
}
