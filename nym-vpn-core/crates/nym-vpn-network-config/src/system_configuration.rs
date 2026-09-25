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
    // Absent on discovery caches written before this field existed.
    #[serde(default = "default_app_update_policy")]
    pub app_update_policy: String,
}

fn default_app_update_policy() -> String {
    "dismissible".to_owned()
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

#[cfg(test)]
mod tests {
    use super::SystemConfiguration;

    #[test]
    fn app_update_policy_missing_from_cached_discovery_is_dismissible() {
        let json = r#"{
            "mix_thresholds": {"high": 75, "medium": 50, "low": 25},
            "wg_thresholds": {"high": 75, "medium": 50, "low": 25},
            "statistics_api": null,
            "min_supported_app_versions": null
        }"#;
        let parsed: SystemConfiguration = serde_json::from_str(json).expect("old cache shape");
        assert_eq!(parsed.app_update_policy, "dismissible");
    }
}
