// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[derive(Debug, Clone)]
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
pub struct UserAgent {
    // The name of the application
    // Example: nym-vpnd
    #[cfg_attr(feature = "cbor", n(0))]
    pub application: String,

    // The version
    #[cfg_attr(feature = "cbor", n(1))]
    pub version: String,

    // The platform triple
    // Example: x86_64-unknown-linux-gnu
    #[cfg_attr(feature = "cbor", n(2))]
    pub platform: String,

    // The git commit hash
    #[cfg_attr(feature = "cbor", n(3))]
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
