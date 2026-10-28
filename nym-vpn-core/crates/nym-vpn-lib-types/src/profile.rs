// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
pub enum Profile {
    // 2 hop configuration with automatic selection with different jurisdictions.
    #[cfg_attr(feature = "cbor", n(0))]
    Safest,
    // 5 hop configuration with automatic selection with different jurisdictions.
    #[cfg_attr(feature = "cbor", n(1))]
    MostPrivate,
    // 2 hop configuration with automatic selection regardless of jurisdiction.
    #[cfg_attr(feature = "cbor", n(2))]
    Fastest,
    // 2 hop configuration with random servers.
    #[cfg_attr(feature = "cbor", n(3))]
    Random,
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct ProfileOptions {
    #[cfg_attr(feature = "cbor", n(0))]
    pub profile: Profile,
}
