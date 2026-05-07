// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum GatewaySelectionAlgorithm {
    /// Select gateways explicitly using the entry and exit selectors.
    Explicit,

    /// Select gateways explicitly using the exit selector and automatically finding an entry gateway.
    AutoEntryExplicitExit,

    #[default]
    /// Select gateways by automatically finding an entry and an exit gateway.
    /// The hop mode is also automatically set to 2-hop
    Auto,
}

impl fmt::Display for GatewaySelectionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Explicit => "Explicit",
            Self::AutoEntryExplicitExit => "Explicit for exit",
            Self::Auto => "Auto",
        };
        write!(f, "{s}")
    }
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
pub struct GatewaySelectionAlgorithmConfig {
    /// Whether selection algorithm uses geo-location is enabled.
    pub enable_geo_location: bool,
    /// The gateway selection algorithm that should be used.
    pub gateway_selection_algorithm: GatewaySelectionAlgorithm,
}

impl Default for GatewaySelectionAlgorithmConfig {
    fn default() -> Self {
        Self {
            enable_geo_location: true,
            gateway_selection_algorithm: Default::default(),
        }
    }
}

impl fmt::Display for GatewaySelectionAlgorithmConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geo location enabled: {}; ", self.enable_geo_location)?;
        write!(
            f,
            "gateway selection algorithm: {}",
            self.gateway_selection_algorithm
        )
    }
}
