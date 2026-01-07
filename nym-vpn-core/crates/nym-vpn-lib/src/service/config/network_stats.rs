// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) mod v1 {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub struct NetworkStatisticsConfig {
        pub enabled: bool,
        pub allow_disconnected: bool,
    }

    impl From<NetworkStatisticsConfig> for nym_vpn_lib_types::NetworkStatisticsConfig {
        fn from(value: NetworkStatisticsConfig) -> Self {
            Self {
                enabled: value.enabled,
                allow_disconnected: value.allow_disconnected,
            }
        }
    }

    // This is only required for the latest version of the external entry point representation
    impl From<&nym_vpn_lib_types::NetworkStatisticsConfig> for NetworkStatisticsConfig {
        fn from(value: &nym_vpn_lib_types::NetworkStatisticsConfig) -> Self {
            Self {
                enabled: value.enabled,
                allow_disconnected: value.allow_disconnected,
            }
        }
    }
}
