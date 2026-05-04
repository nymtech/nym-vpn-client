// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
pub(crate) mod v9 {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum GatewaySelectionAlgorithm {
        Explicit,
        AutoEntryExplicitExit,
        Auto,
    }

    impl From<nym_vpn_lib_types::GatewaySelectionAlgorithm> for GatewaySelectionAlgorithm {
        fn from(value: nym_vpn_lib_types::GatewaySelectionAlgorithm) -> Self {
            match value {
                nym_vpn_lib_types::GatewaySelectionAlgorithm::Explicit => Self::Explicit,
                nym_vpn_lib_types::GatewaySelectionAlgorithm::AutoEntryExplicitExit => {
                    Self::AutoEntryExplicitExit
                }
                nym_vpn_lib_types::GatewaySelectionAlgorithm::Auto => Self::Auto,
            }
        }
    }

    impl From<GatewaySelectionAlgorithm> for nym_vpn_lib_types::GatewaySelectionAlgorithm {
        fn from(value: GatewaySelectionAlgorithm) -> Self {
            match value {
                GatewaySelectionAlgorithm::Explicit => Self::Explicit,
                GatewaySelectionAlgorithm::AutoEntryExplicitExit => Self::AutoEntryExplicitExit,
                GatewaySelectionAlgorithm::Auto => Self::Auto,
            }
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub struct GatewaySelectionAlgorithmConfig {
        pub enable_geo_location: bool,
        pub gateway_selection_algorithm: GatewaySelectionAlgorithm,
    }

    impl From<&nym_vpn_lib_types::GatewaySelectionAlgorithmConfig> for GatewaySelectionAlgorithmConfig {
        fn from(value: &nym_vpn_lib_types::GatewaySelectionAlgorithmConfig) -> Self {
            Self {
                enable_geo_location: value.enable_geo_location,
                gateway_selection_algorithm: value.gateway_selection_algorithm.into(),
            }
        }
    }

    impl From<GatewaySelectionAlgorithmConfig> for nym_vpn_lib_types::GatewaySelectionAlgorithmConfig {
        fn from(value: GatewaySelectionAlgorithmConfig) -> Self {
            Self {
                enable_geo_location: value.enable_geo_location,
                gateway_selection_algorithm: value.gateway_selection_algorithm.into(),
            }
        }
    }
}
