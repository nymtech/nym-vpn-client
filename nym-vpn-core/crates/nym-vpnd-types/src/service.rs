// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{EntryPoint, ExitPoint};
use nym_sdk::UserAgent;
use nym_vpn_network_config::{NymNetwork, NymVpnNetwork};
use std::net::IpAddr;
use time::OffsetDateTime;

/// The target tunnel state.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TargetState {
    /// Unsecure the device.
    Unsecured,

    /// Secure the device.
    Secured,
}

impl std::fmt::Display for TargetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TargetState::Unsecured => "Unsecured",
            TargetState::Secured => "Secured",
        };
        write!(f, "{s}")
    }
}

// Deprecated
#[derive(Debug)]
pub struct ConnectArgs {
    pub entry: Option<EntryPoint>,
    pub exit: Option<ExitPoint>,
    pub options: ConnectOptions,
}

// Deprecated
#[derive(Default, Debug, Clone)]
pub struct ConnectOptions {
    pub dns: Option<IpAddr>,
    pub disable_ipv6: bool,
    pub enable_two_hop: bool,
    pub netstack: bool,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub enable_credentials_mode: bool,
    pub user_agent: Option<UserAgent>,
}

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
