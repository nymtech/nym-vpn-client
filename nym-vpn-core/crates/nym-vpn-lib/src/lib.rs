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

use std::{net::IpAddr, sync::LazyLock};

use hickory_resolver::config::NameServerConfigGroup;
use itertools::Itertools;
use nym_platform_metadata::version;
use tracing::info;

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

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use crate::platform::swift;
pub use crate::{
    error::{Error, GatewayDirectoryError},
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

pub struct SysInfo {
    pub os_version: String,
    pub arch: String,
    pub extra: Vec<String>,
}

impl SysInfo {
    pub fn new() -> Self {
        let os_version = version();
        let arch = std::env::consts::ARCH.to_string();
        let extra_metadata = nym_platform_metadata::extra_metadata()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>();

        SysInfo {
            os_version,
            arch,
            extra: extra_metadata,
        }
    }

    pub fn display(&self, print_extra: bool) {
        info!("os version: {}", self.os_version);
        info!("os arch: {}", self.arch);
        if print_extra {
            for info in &self.extra {
                info!("os {info}");
            }
        }
    }

    pub fn raw_display(&self, print_extra: bool) {
        println!("os version: {}", self.os_version);
        println!("os arch: {}", self.arch);
        if print_extra {
            for info in &self.extra {
                println!("os {info}");
            }
        }
    }
}

impl Default for SysInfo {
    fn default() -> Self {
        Self::new()
    }
}
