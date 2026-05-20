// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
pub(crate) mod v10 {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub struct GatewayIndependence {
        pub different_node_family: bool,
        pub different_asn: bool,
    }

    impl From<&nym_vpn_lib_types::GatewayIndependence> for GatewayIndependence {
        fn from(value: &nym_vpn_lib_types::GatewayIndependence) -> Self {
            Self {
                different_node_family: value.different_node_family,
                different_asn: value.different_asn,
            }
        }
    }

    impl From<GatewayIndependence> for nym_vpn_lib_types::GatewayIndependence {
        fn from(value: GatewayIndependence) -> Self {
            Self {
                different_node_family: value.different_node_family,
                different_asn: value.different_asn,
            }
        }
    }
}
