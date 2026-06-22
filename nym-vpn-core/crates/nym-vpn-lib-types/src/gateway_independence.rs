// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct GatewayIndependence {
    pub enable_notifications: bool,
    pub different_node_family: bool,
    pub different_asn: bool,
    pub different_subnet: bool,
}

impl GatewayIndependence {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.different_node_family = enabled;
        self.different_asn = enabled;
        self.different_subnet = enabled;
    }

    pub fn active(&self) -> bool {
        self.different_node_family || self.different_asn || self.different_subnet
    }

    pub fn full_disabled(&self) -> bool {
        !self.different_node_family && !self.different_asn && !self.different_subnet
    }

    pub fn full_enabled(&self) -> bool {
        self.different_node_family && self.different_asn && self.different_subnet
    }
}

impl Default for GatewayIndependence {
    fn default() -> Self {
        Self {
            enable_notifications: true,
            different_node_family: true,
            different_asn: true,
            different_subnet: true,
        }
    }
}

impl fmt::Display for GatewayIndependence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "enabled notifications: {}; ", self.enable_notifications)?;
        write!(
            f,
            "different node family: {}; different asn: {}; different subnet: {}",
            self.different_node_family, self.different_asn, self.different_subnet
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_both_criteria_active() {
        let gi = GatewayIndependence::default();
        assert!(gi.different_node_family);
        assert!(gi.different_asn);
    }

    #[test]
    fn deactivated_has_no_criteria() {
        let gi = GatewayIndependence {
            different_node_family: false,
            different_asn: false,
            different_subnet: false,
            ..Default::default()
        };
        assert!(!gi.different_node_family);
        assert!(!gi.different_asn);
    }

    #[test]
    fn active_returns_true_for_default() {
        assert!(GatewayIndependence::default().active());
    }

    #[test]
    fn active_returns_false_when_fully_deactivated() {
        assert!(
            !GatewayIndependence {
                different_node_family: false,
                different_asn: false,
                different_subnet: false,
                ..Default::default()
            }
            .active()
        );
    }

    #[test]
    fn active_returns_true_with_only_asn_enabled() {
        let gi = GatewayIndependence {
            different_asn: true,
            different_node_family: false,
            different_subnet: false,
            ..Default::default()
        };
        assert!(gi.active());
    }

    #[test]
    fn active_returns_true_with_only_family_enabled() {
        let gi = GatewayIndependence {
            different_asn: false,
            different_node_family: true,
            different_subnet: false,
            ..Default::default()
        };
        assert!(gi.active());
    }

    #[test]
    fn active_returns_true_with_only_subnet_enabled() {
        let gi = GatewayIndependence {
            different_asn: false,
            different_node_family: false,
            different_subnet: true,
            ..Default::default()
        };
        assert!(gi.active());
    }
}
