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
pub enum Profile {
    // 2 hop configuration with automatic selection with different jurisdictions.
    Safest,
    // 5 hop configuration with automatic selection with different jurisdictions.
    MostPrivate,
    // 2 hop configuration with automatic selection regardless of jurisdiction.
    Fastest,
    // 2 hop configuration with random servers.
    Random,
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct ProfileOptions {
    pub profile: Profile,
}
