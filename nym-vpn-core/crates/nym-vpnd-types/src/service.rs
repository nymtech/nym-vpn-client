// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{EntryPoint, ExitPoint};
use nym_sdk::UserAgent;
use nym_vpn_network_config::{NymNetwork, NymVpnNetwork};
use std::{fmt, net::IpAddr};
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VpnServiceConfig {
    pub entry_point: EntryPoint,
    pub exit_point: ExitPoint,
    pub dns: Option<IpAddr>,
    pub allow_lan: bool,
    pub disable_ipv6: bool,
    pub enable_two_hop: bool,
    pub netstack: bool,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub min_mixnode_performance: Option<u8>,
    pub min_gateway_mixnet_performance: Option<u8>,
    pub min_gateway_vpn_performance: Option<u8>,
}

impl fmt::Display for VpnServiceConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "entry point: {}, exit point: {}",
            self.entry_point, self.exit_point
        )
    }
}

impl Default for VpnServiceConfig {
    fn default() -> Self {
        Self {
            entry_point: EntryPoint::Random,
            exit_point: ExitPoint::Random,
            dns: None,
            disable_ipv6: false,
            enable_two_hop: false,
            netstack: false,
            allow_lan: false,
            disable_poisson_rate: false,
            disable_background_cover_traffic: false,
            min_mixnode_performance: None,
            min_gateway_mixnet_performance: None,
            min_gateway_vpn_performance: None,
        }
    }
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
