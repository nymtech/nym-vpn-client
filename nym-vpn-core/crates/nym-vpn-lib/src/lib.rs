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
pub use nym_config;
pub use nym_gateway_directory as gateway_directory;
pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::{
    UserAgent,
    mixnet::{NodeIdentity, Recipient, StoragePaths},
};

// Re-exports used by new_user_agent macros
#[doc(hidden)]
pub use nym_bin_common;
#[doc(hidden)]
pub use nym_platform_metadata;

pub use crate::{
    error::GatewayDirectoryError,
    mixnet::{
        DEFAULT_MIN_GATEWAY_PERFORMANCE, DEFAULT_MIN_MIXNODE_PERFORMANCE, MixnetError,
        VpnTopologyProvider,
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

/// Macro that creates `UserAgent` from compiled in vergen metadata.
#[macro_export]
macro_rules! new_user_agent {
    () => {{
        let bin_info = $crate::nym_bin_common::bin_info_local_vergen!();
        let sys_info = $crate::nym_platform_metadata::SysInfo::new();
        let platform = format!(
            "{}; {}; {}",
            sys_info.system_name, sys_info.os_version, sys_info.arch
        );
        $crate::UserAgent {
            application: bin_info.binary_name.to_string(),
            version: bin_info.build_version.to_string(),
            platform,
            git_commit: bin_info.commit_sha.to_string(),
        }
    }};
}
