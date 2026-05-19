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
    pub different_node_family: bool,
    pub different_asn: bool,
}

impl GatewayIndependence {
    pub fn new_deactivated() -> Self {
        Self {
            different_node_family: false,
            different_asn: false,
        }
    }

    pub fn active(&self) -> bool {
        self.different_node_family || self.different_asn
    }
}

impl Default for GatewayIndependence {
    fn default() -> Self {
        Self {
            different_node_family: true,
            different_asn: true,
        }
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
        let gi = GatewayIndependence::new_deactivated();
        assert!(!gi.different_node_family);
        assert!(!gi.different_asn);
    }

    #[test]
    fn active_returns_true_for_default() {
        assert!(GatewayIndependence::default().active());
    }

    #[test]
    fn active_returns_false_when_fully_deactivated() {
        assert!(!GatewayIndependence::new_deactivated().active());
    }

    #[test]
    fn active_returns_true_with_only_asn_enabled() {
        let gi = GatewayIndependence {
            different_asn: true,
            different_node_family: false,
        };
        assert!(gi.active());
    }

    #[test]
    fn active_returns_true_with_only_family_enabled() {
        let gi = GatewayIndependence {
            different_asn: false,
            different_node_family: true,
        };
        assert!(gi.active());
    }
}

impl fmt::Display for GatewayIndependence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "different node family: {}; ", self.different_node_family)?;
        write!(f, "different ASN: {}", self.different_asn)
    }
}
