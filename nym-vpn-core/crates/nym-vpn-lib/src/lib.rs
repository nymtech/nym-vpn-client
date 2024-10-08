// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

pub mod credentials;
pub mod storage;
pub mod util;

mod error;
mod mixnet;
#[cfg(any(target_os = "ios", target_os = "android"))]
mod mobile;
mod platform;
pub mod tunnel_state_machine;
mod uniffi_custom_impls;
mod wg_config;

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};

// Re-export some our nym dependencies
pub use nym_authenticator_client::Error as AuthenticatorClientError;
pub use nym_config;
pub use nym_connection_monitor as connection_monitor;
pub use nym_credential_storage_pre_ecash::error::StorageError as CredentialStorageError;
pub use nym_gateway_directory as gateway_directory;
pub use nym_id_pre_ecash::error::NymIdError;
pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::{
    mixnet::{NodeIdentity, Recipient, StoragePaths},
    UserAgent,
};
pub use nym_task::{
    manager::{SentStatus, TaskStatus},
    StatusReceiver,
};
pub use nym_wg_gateway_client as wg_gateway_client;

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use crate::platform::swift;
pub use crate::{
    error::{Error, GatewayDirectoryError},
    mixnet::MixnetError,
};

use nym_gateway_directory::{Config as GatewayDirectoryConfig, EntryPoint, ExitPoint};

pub const DEFAULT_DNS_SERVERS: [IpAddr; 4] = [
    IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)),
    IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111)),
    IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1001)),
];

#[derive(Clone, Debug)]
pub struct GenericNymVpnConfig {
    pub mixnet_client_config: MixnetClientConfig,

    /// Path to the data directory, where keys reside.
    pub data_path: Option<PathBuf>,

    /// Gateway configuration
    pub gateway_config: GatewayDirectoryConfig,

    /// Mixnet public ID of the entry gateway.
    pub entry_point: EntryPoint,

    /// Mixnet recipient address.
    pub exit_point: ExitPoint,

    /// The IP addresses of the TUN device.
    pub nym_ips: Option<IpPair>,

    /// The MTU of the TUN device.
    pub nym_mtu: Option<u16>,

    /// The DNS server to use
    pub dns: Option<IpAddr>,

    /// Disable routing all traffic through the VPN TUN device.
    pub disable_routing: bool,

    /// The user agent to use for HTTP requests. This includes client name, version, platform and
    /// git commit hash.
    pub user_agent: Option<UserAgent>,
}

#[derive(Clone, Debug)]
pub struct MixnetClientConfig {
    /// Enable Poission process rate limiting of outbound traffic.
    pub enable_poisson_rate: bool,

    /// Disable constant rate background loop cover traffic
    pub disable_background_cover_traffic: bool,

    /// Enable the credentials mode between the client and the entry gateway.
    pub enable_credentials_mode: bool,

    /// The minimum performance of mixnodes to use.
    pub min_mixnode_performance: Option<u8>,

    /// The minimum performance of gateways to use.
    pub min_gateway_performance: Option<u8>,
}

impl MixnetClientConfig {
    pub fn mixnet_default() -> Self {
        Self {
            enable_poisson_rate: false,
            disable_background_cover_traffic: false,
            enable_credentials_mode: false,
            min_mixnode_performance: None,
            min_gateway_performance: None,
        }
    }

    pub fn wireguard_default() -> Self {
        Self {
            enable_poisson_rate: false,
            disable_background_cover_traffic: true,
            enable_credentials_mode: false,
            min_mixnode_performance: None,
            min_gateway_performance: None,
        }
    }
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
