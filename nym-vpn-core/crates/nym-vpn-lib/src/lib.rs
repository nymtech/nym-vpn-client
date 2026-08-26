// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod paths;
pub mod storage;

mod adblocker;
mod bandwidth_monitor;
pub mod cache_refresh;
pub mod config;
mod dns_filter;
pub mod logging;
mod mixnet;
pub mod privy;
#[cfg(not(target_os = "android"))]
mod resolver;
pub mod sentry;
pub mod service;
#[cfg(not(target_os = "ios"))]
pub(crate) mod socks5_proxy;
mod tunnel_health;
pub mod tunnel_provider;
pub mod tunnel_state_machine;
mod wg_config;

use std::sync::LazyLock;

use hickory_resolver::config::{CLOUDFLARE, NameServerConfig, QUAD9};
#[cfg(target_os = "windows")]
pub use nym_split_tunnel::install_driver_service as install_split_tunnel_driver_service;
#[cfg(target_os = "windows")]
pub use nym_split_tunnel::uninstall_driver_service as uninstall_split_tunnel_driver_service;

// Re-export some our nym dependencies
pub use nym_config;
pub use nym_gateway_directory as gateway_directory;
pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::{
    UserAgent,
    mixnet::{NodeIdentity, Recipient, StoragePaths},
};

pub use crate::{
    mixnet::{
        DEFAULT_MIN_GATEWAY_PERFORMANCE, DEFAULT_MIN_MIXNODE_PERFORMANCE, MixnetError,
        VpnTopologyProvider, VpnTopologyService, VpnTopologyServiceError, VpnTopologyServiceHandle,
    },
    tunnel_state_machine::tunnel::gateway_provider::GatewayProviderError,
};

/// Default DNS servers.
static DEFAULT_DNS_SERVERS_CONFIG: LazyLock<Vec<NameServerConfig>> = LazyLock::new(|| {
    QUAD9
        .tls()
        .chain(QUAD9.https())
        .chain(CLOUDFLARE.tls())
        .chain(CLOUDFLARE.https())
        .filter(|ns| {
            // Exclude IPv6 addresses due to reliability issues
            ns.ip.is_ipv4()
        })
        .collect::<Vec<_>>()
});

/// Routing table id used for routing all traffic through the tunnel.
#[cfg(target_os = "linux")]
pub const TUNNEL_TABLE_ID: u32 = 0x14d;

/// Firewall mark used for marking traffic that should bypass the tunnel.
#[cfg(target_os = "linux")]
pub use nym_firewall_config::TUNNEL_FWMARK;

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
