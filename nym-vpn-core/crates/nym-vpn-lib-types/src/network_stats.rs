// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct NetworkStatisticsConfig {
    /// Is stats reporting enabled
    pub enabled: bool,

    // Allow sending reports even with a not connected tunnel
    // This actually automatically takes the correct reporting channel :
    // - If the tunnel is not connected, it will go directly to the endpoint
    // - If the tunnel is connected in mixnet mode, it will go through the mixnet
    // - If the tunnel is connected in wireguard mode, it will go through wireguard
    // We thus need this option to allow users to report even if they can't connect (i.e. if they're being censored)
    /// Allow statistics sending, even if not connected to the mixnet/vpn
    pub allow_disconnected: bool,
}

impl Default for NetworkStatisticsConfig {
    fn default() -> Self {
        NetworkStatisticsConfig {
            enabled: true,
            allow_disconnected: false,
        }
    }
}

impl fmt::Display for NetworkStatisticsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "enabled : {}, allow_disconnected : {}",
            self.enabled, self.allow_disconnected
        )?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct NetworkStatisticsIdentity {
    /// Seed used to generate the id
    pub seed: String,

    /// Actual ID used
    pub id: String,
}
