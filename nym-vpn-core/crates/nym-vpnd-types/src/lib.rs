// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod gateway;
pub mod log_path;
pub mod service;

use std::net::IpAddr;

use nym_vpn_lib::{
    UserAgent,
    gateway_directory::{EntryPoint, ExitPoint, Percent},
};

#[derive(Debug)]
pub struct ConnectArgs {
    pub entry: Option<EntryPoint>,
    pub exit: Option<ExitPoint>,
    pub options: ConnectOptions,
}

#[derive(Default, Debug, Clone)]
pub struct ConnectOptions {
    pub dns: Option<IpAddr>,
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
