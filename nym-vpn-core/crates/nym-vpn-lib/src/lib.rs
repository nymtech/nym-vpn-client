// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

pub mod storage;
pub mod util;

mod error;
mod event;
mod mixnet;
#[cfg(any(target_os = "ios", target_os = "android"))]
mod mobile;
mod platform;
mod routing;
mod tunnel;
mod tunnel_setup;
pub mod tunnel_state_machine;
mod uniffi_custom_impls;
mod vpn;
mod wg_config;
mod wireguard_config;
mod wireguard_setup;

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
    manager::{SentStatus, TaskStatus},
    StatusReceiver,
};
pub use nym_wg_gateway_client as wg_gateway_client;

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use crate::platform::swift;
pub use crate::{
    error::{Error, GatewayDirectoryError, SetupMixTunnelError, SetupWgTunnelError},
    event::WgTunnelErrorEvent,
    mixnet::MixnetError,
    vpn::{
        spawn_nym_vpn, spawn_nym_vpn_with_new_runtime, GenericNymVpnConfig, MixnetClientConfig,
        NymVpn, NymVpnCtrlMessage, NymVpnExitStatusMessage, NymVpnHandle, NymVpnStatusMessage,
        SpecificVpn,
    },
};

pub const DEFAULT_DNS_SERVERS: [IpAddr; 4] = [
    IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)),
    IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111)),
    IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1001)),
];
