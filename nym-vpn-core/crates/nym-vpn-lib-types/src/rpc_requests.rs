// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use nym_gateway_directory::{EntryPoint, ExitPoint, GatewayType};
use nym_sdk::UserAgent;

#[derive(Debug)]
pub struct ListGatewaysOptions {
    pub gw_type: GatewayType,
    #[allow(unused)]
    pub user_agent: Option<UserAgent>,
}

#[derive(zeroize::Zeroize)]
pub enum StoreAccountRequest {
    Vpn { mnemonic: String },
    Decentralised { mnemonic: String },
}

impl std::fmt::Debug for StoreAccountRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreVpnAccountRequest")
            .field("mnemonic", &"[redacted]")
            .finish()
    }
}

#[derive(Debug)]
pub struct AccountCommandResponse {
    pub error: Option<crate::AccountCommandError>,
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
    pub enable_bridges: bool,
    pub netstack: bool,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub enable_credentials_mode: bool,
    pub user_agent: Option<UserAgent>,
}
