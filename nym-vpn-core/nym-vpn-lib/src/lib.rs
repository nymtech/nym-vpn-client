// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

pub mod credentials;
pub mod storage;
pub mod util;

mod bandwidth_controller;
mod error;
mod mixnet;
mod platform;
mod routing;
mod tunnel;
mod tunnel_setup;
mod uniffi_custom_impls;
mod vpn;
mod wireguard_config;
mod wireguard_setup;

// Re-export some our nym dependencies
pub use nym_bin_common as bin_common;
pub use nym_config;
pub use nym_connection_monitor as connection_monitor;
pub use nym_gateway_directory as gateway_directory;
pub use nym_wg_gateway_client as wg_gateway_client;

pub use nym_credential_storage_pre_ecash::error::StorageError as CredentialStorageError;
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

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use crate::platform::swift;
pub use crate::{
    error::{Error, GatewayDirectoryError, MixnetError},
    vpn::{
        spawn_nym_vpn, spawn_nym_vpn_with_new_runtime, GenericNymVpnConfig, MixnetClientConfig,
        NymVpn, NymVpnCtrlMessage, NymVpnExitStatusMessage, NymVpnHandle, NymVpnStatusMessage,
        SpecificVpn,
    },
};
