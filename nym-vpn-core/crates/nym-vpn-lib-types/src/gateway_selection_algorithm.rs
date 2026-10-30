// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

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
pub struct GatewaySelectionAlgorithmConfig {
    /// Whether selection algorithm uses geo-location is enabled.
    #[cfg_attr(feature = "cbor", n(0))]
    pub enable_geo_location: bool,
}

impl GatewaySelectionAlgorithmConfig {
    pub fn new(enable_geo_location: bool) -> Self {
        Self {
            enable_geo_location,
        }
    }
}

impl Default for GatewaySelectionAlgorithmConfig {
    fn default() -> Self {
        Self {
            enable_geo_location: true,
        }
    }
}

impl fmt::Display for GatewaySelectionAlgorithmConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geo location enabled: {}; ", self.enable_geo_location)
    }
}
