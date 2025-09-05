// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod gateway;
pub mod log_path;
pub mod service;

use std::net::IpAddr;

use nym_gateway_directory::{EntryPoint, ExitPoint, GatewayType, Percent};
use nym_sdk::UserAgent;

#[derive(Debug)]
pub struct ConnectArgs {
    pub entry: Option<EntryPoint>,
    pub exit: Option<ExitPoint>,
    pub options: ConnectOptions,
}

#[derive(Default, Debug, Clone)]
pub struct ConnectOptions {
    pub dns: Option<IpAddr>,
    pub disable_ipv6: bool,
    pub enable_two_hop: bool,
    pub netstack: bool,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub enable_credentials_mode: bool,
    pub min_mixnode_performance: Option<Percent>,
    pub min_gateway_mixnet_performance: Option<Percent>,
    pub min_gateway_vpn_performance: Option<Percent>,
    pub user_agent: Option<UserAgent>,
}

#[derive(Debug)]
pub struct ListGatewaysOptions {
    pub gw_type: GatewayType,
    #[allow(unused)]
    pub user_agent: Option<UserAgent>,
}

#[derive(Debug)]
pub struct ListCountriesOptions {
    pub gw_type: GatewayType,
    #[allow(unused)]
    pub user_agent: Option<UserAgent>,
}

#[derive(zeroize::Zeroize)]
pub struct StoreAccountRequest {
    pub mnemonic: String,
}

impl std::fmt::Debug for StoreAccountRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreAccountRequest")
            .field("mnemonic", &"[redacted]")
            .finish()
    }
}

pub struct AccountCommandResponse {
    pub error: Option<nym_vpn_lib_types::AccountCommandError>,
}
