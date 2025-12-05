// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use crate::{GatewayType, UserAgent};

#[derive(Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct ListGatewaysOptions {
    pub gw_type: GatewayType,
    #[allow(unused)]
    pub user_agent: Option<UserAgent>,
}

#[derive(zeroize::Zeroize)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum LoginSecret {
    Mnemonic(String),
    PrivyHexSignature(String),
}

#[derive(zeroize::Zeroize)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
pub enum StoreAccountRequest {
    Vpn { secret: LoginSecret },
    Decentralised { secret: LoginSecret },
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
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
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
