// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_network_config::{NymNetwork, NymVpnNetwork};
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub struct VpnServiceInfo {
    pub version: String,
    pub build_timestamp: Option<OffsetDateTime>,
    pub triple: String,
    pub platform: String,
    pub git_commit: String,
    pub nym_network: NymNetwork,
    pub nym_vpn_network: NymVpnNetwork,
}

// Failure to initiate the connect
#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceConnectError {
    #[error("internal error: {0}")]
    Internal(String),

    #[error("connection attempt cancelled")]
    Cancel,
}

// Failure to initiate the disconnect
#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceDisconnectError {
    #[error("internal error: {0}")]
    Internal(String),
}
