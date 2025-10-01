// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use crate::{EntryPoint, ExitPoint, GatewayType};
use nym_validator_client::nyxd;

#[derive(Debug, Clone)]
pub struct UserAgent {
    // The name of the application
    // Example: nym-vpnd
    pub application: String,

    // The version
    pub version: String,

    // The platform triple
    // Example: x86_64-unknown-linux-gnu
    pub platform: String,

    // The git commit hash
    pub git_commit: String,
}

impl std::str::FromStr for UserAgent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 4 {
            return Err(format!("invalid user agent string: {s}"));
        }

        Ok(UserAgent {
            application: parts[0].to_string(),
            version: parts[1].to_string(),
            platform: parts[2].to_string(),
            git_commit: parts[3].to_string(),
        })
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<UserAgent> for nym_sdk::UserAgent {
    fn from(value: UserAgent) -> Self {
        nym_sdk::UserAgent {
            application: value.application,
            version: value.version,
            platform: value.platform,
            git_commit: value.git_commit,
        }
    }
}

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
pub struct DecentralisedObtainTicketbooksRequest {
    pub amount: u64,
}

#[derive(Debug)]
pub struct AccountCommandResponse {
    pub error: Option<crate::AccountCommandError>,
}

#[derive(Debug)]
pub struct AccountBalanceResponse {
    pub result: Result<Vec<nyxd::Coin>, crate::AccountCommandError>,
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
