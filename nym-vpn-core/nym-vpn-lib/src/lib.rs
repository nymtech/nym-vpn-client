// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

pub mod credentials;
pub mod storage;
pub mod util;
pub mod vpn;

mod bandwidth_controller;
mod config;
mod error;
mod mixnet;
mod platform;
mod routing;
mod tunnel;
mod tunnel_setup;
mod uniffi_custom_impls;
mod wg_gateway_client;
mod wireguard_setup;

pub use nym_bin_common;
pub use nym_config;
pub use nym_connection_monitor as connection_monitor;
pub use nym_credential_storage_pre_ecash as credential_storage_pre_ecash;
pub use nym_gateway_directory as gateway_directory;
pub use nym_id_pre_ecash as id_pre_ecash;

pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::mixnet::{NodeIdentity, Recipient, StoragePaths};
pub use nym_sdk::UserAgent;
pub use nym_task::{
    manager::{SentStatus, TaskStatus},
    StatusReceiver,
};

pub use crate::error::{Error, GatewayDirectoryError, MixnetError};
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use crate::platform::swift;
