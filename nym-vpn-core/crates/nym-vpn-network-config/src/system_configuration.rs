// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::response::SystemConfigurationResponse;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemConfiguration {
    pub score_thresholds: ScoreThresholds,
}

impl From<SystemConfigurationResponse> for SystemConfiguration {
    fn from(value: SystemConfigurationResponse) -> Self {
        SystemConfiguration {
            score_thresholds: ScoreThresholds {
                high: value.high,
                medium: value.medium,
                low: value.low,
            },
        }
    }
}

impl fmt::Display for SystemConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.score_thresholds)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreThresholds {
    pub high: u8,
    pub medium: u8,
    pub low: u8,
}
