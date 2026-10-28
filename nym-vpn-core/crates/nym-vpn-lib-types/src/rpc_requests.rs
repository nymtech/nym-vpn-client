// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[cfg(feature = "nym-type-conversions")]
use crate::{Mnemonic, StorableAccount, StoredAccountMode};

use crate::{GatewayType, UserAgent};

#[derive(Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct ListGatewaysOptions {
    #[cfg_attr(feature = "cbor", n(0))]
    pub gw_type: GatewayType,
    #[allow(unused)]
    #[cfg_attr(feature = "cbor", n(1))]
    pub user_agent: Option<UserAgent>,
}

#[derive(zeroize::Zeroize)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum StoreAccountRequest {
    #[cfg_attr(feature = "cbor", n(0))]
    Vpn {
        #[cfg_attr(feature = "cbor", n(0))]
        mnemonic: String,
    },
    #[cfg_attr(feature = "cbor", n(1))]
    Privy {
        #[cfg_attr(feature = "cbor", n(0))]
        mnemonic: String,
    },
    #[cfg_attr(feature = "cbor", n(2))]
    Decentralised {
        #[cfg_attr(feature = "cbor", n(0))]
        mnemonic: String,
    },
}

impl StoreAccountRequest {
    pub fn centralised(&self) -> bool {
        !matches!(self, Self::Decentralised { .. })
    }
}

impl std::fmt::Debug for StoreAccountRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreVpnAccountRequest")
            .field("mnemonic", &"[redacted]")
            .finish()
    }
}

#[cfg(feature = "nym-type-conversions")]
impl TryFrom<StoreAccountRequest> for StorableAccount {
    type Error = bip39::Error;

    fn try_from(value: StoreAccountRequest) -> Result<Self, Self::Error> {
        Ok(match value {
            StoreAccountRequest::Vpn { mnemonic } => {
                StorableAccount::new(Mnemonic::parse(&mnemonic)?, StoredAccountMode::Api)
            }
            StoreAccountRequest::Privy { mnemonic } => {
                StorableAccount::new(Mnemonic::parse(&mnemonic)?, StoredAccountMode::Privy)
            }
            StoreAccountRequest::Decentralised { mnemonic } => StorableAccount::new(
                Mnemonic::parse(&mnemonic)?,
                StoredAccountMode::Decentralised,
            ),
        })
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct AccountCommandResponse {
    #[cfg_attr(feature = "cbor", n(0))]
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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct Coin {
    #[cfg_attr(feature = "cbor", n(0))]
    pub amount: u128,
    #[cfg_attr(feature = "cbor", n(1))]
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
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct AccountBalanceResponse {
    #[cfg_attr(feature = "cbor", n(0))]
    pub result: Result<Vec<Coin>, crate::AccountCommandError>,
}
