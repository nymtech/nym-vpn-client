// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use crate::{EntryPoint, ExitPoint, GatewayType};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
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
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct ListGatewaysOptions {
    pub gw_type: GatewayType,
    #[allow(unused)]
    pub user_agent: Option<UserAgent>,
}

#[derive(zeroize::Zeroize)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
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
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct DecentralisedObtainTicketbooksRequest {
    pub amount: u64,
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct AccountCommandResponse {
    pub error: Option<crate::AccountCommandError>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct Coin {
    pub amount: u128,
    pub denom: String,
}

impl Coin {
    pub fn new(amount: u128, denom: String) -> Self {
        Self { amount, denom }
    }
}

impl std::fmt::Display for Coin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.amount, self.denom)
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_validator_client::nyxd::Coin> for Coin {
    fn from(value: nym_validator_client::nyxd::Coin) -> Self {
        Self {
            amount: value.amount,
            denom: value.denom,
        }
    }
}

// todo: figure out how to pass Result over uniffi
#[derive(Debug)]
//#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct AccountBalanceResponse {
    pub result: Result<Vec<Coin>, crate::AccountCommandError>,
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
