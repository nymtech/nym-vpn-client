// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod gateway;
pub mod log_path;
pub mod service;

use nym_gateway_directory::GatewayType;
use nym_sdk::UserAgent;

#[derive(Debug)]
pub struct ListGatewaysOptions {
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
