// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::response::SystemConfigurationResponse;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemConfiguration {
    pub mix_thresholds: ScoreThresholds,
    pub wg_thresholds: ScoreThresholds,
}

impl From<SystemConfigurationResponse> for SystemConfiguration {
    fn from(value: SystemConfigurationResponse) -> Self {
        SystemConfiguration {
            mix_thresholds: ScoreThresholds {
                high: value.mix_thresholds.high,
                medium: value.mix_thresholds.medium,
                low: value.mix_thresholds.low,
            },
            wg_thresholds: ScoreThresholds {
                high: value.wg_thresholds.high,
                medium: value.wg_thresholds.medium,
                low: value.wg_thresholds.low,
            },
        }
    }
}

impl fmt::Display for SystemConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mixnet score thresholds: {:?}\nwireguard score thresholds: {:?}",
            self.mix_thresholds, self.wg_thresholds
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreThresholds {
    pub high: u8,
    pub medium: u8,
    pub low: u8,
}
