// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemConfiguration {
    pub mix_thresholds: ScoreThresholds,
    pub wg_thresholds: ScoreThresholds,
    pub statistics_api: Option<Url>,
    pub min_supported_app_versions: Option<nym_vpn_api_client::NetworkCompatibility>,
}

impl fmt::Display for SystemConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mixnet score thresholds: {:?}\nwireguard score thresholds: {:?}\nstatistics api: {:?}",
            self.mix_thresholds, self.wg_thresholds, self.statistics_api
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreThresholds {
    pub high: u8,
    pub medium: u8,
    pub low: u8,
}
