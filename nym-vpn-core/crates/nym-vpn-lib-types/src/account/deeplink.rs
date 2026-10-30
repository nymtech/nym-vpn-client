// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
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
pub struct AutologinResponse {
    #[cfg_attr(feature = "cbor", n(0))]
    pub url: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub pin_code: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
pub struct GetDeeplinkParams {
    #[cfg_attr(feature = "cbor", n(0))]
    pub client: DeeplinkClient,
    #[cfg_attr(feature = "cbor", n(1))]
    pub locale: String,
    #[cfg_attr(feature = "cbor", n(2))]
    pub kind: DeeplinkKind,
    #[cfg_attr(feature = "cbor", n(3))]
    pub name: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum DeeplinkClient {
    #[cfg_attr(feature = "cbor", n(0))]
    Mobile,
    #[cfg_attr(feature = "cbor", n(1))]
    Desktop,
    #[cfg_attr(feature = "cbor", n(2))]
    Web,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum DeeplinkKind {
    #[cfg_attr(feature = "cbor", n(0))]
    Privy,
    #[cfg_attr(feature = "cbor", n(1))]
    PrivyLink,
    #[cfg_attr(feature = "cbor", n(2))]
    AutologinRenew,
    #[cfg_attr(feature = "cbor", n(3))]
    AutologinView,
    #[cfg_attr(feature = "cbor", n(4))]
    CreateAccount,
}

impl FromStr for DeeplinkKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "privy" => Ok(DeeplinkKind::Privy),
            "privy_link" | "privy-link" | "privylink" => Ok(DeeplinkKind::PrivyLink),
            "autologin_renew" | "autologin-renew" | "autologinrenew" => {
                Ok(DeeplinkKind::AutologinRenew)
            }
            "autologin_view" | "autologin-view" | "autologinview" => {
                Ok(DeeplinkKind::AutologinView)
            }
            "create_account" | "create-account" | "createaccount" => {
                Ok(DeeplinkKind::CreateAccount)
            }
            _ => Err(format!("Unknown deeplink kind: {s}")),
        }
    }
}

impl DeeplinkKind {
    pub fn redirect(&self) -> Option<&str> {
        match self {
            DeeplinkKind::AutologinView => Some("account"),
            DeeplinkKind::AutologinRenew => Some("renew"),
            _ => None,
        }
    }
}
