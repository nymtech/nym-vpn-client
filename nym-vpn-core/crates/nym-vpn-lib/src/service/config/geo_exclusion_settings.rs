// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod v9 {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct GeoExclusionSettings {
        pub enabled: bool,
        pub listen_port: u16,
        pub excluded_countries: Vec<String>,
    }

    impl From<GeoExclusionSettings> for nym_vpn_lib_types::GeoExclusionSettings {
        fn from(value: GeoExclusionSettings) -> Self {
            Self {
                enabled: value.enabled,
                listen_port: value.listen_port,
                excluded_countries: value.excluded_countries,
            }
        }
    }

    impl From<&nym_vpn_lib_types::GeoExclusionSettings> for GeoExclusionSettings {
        fn from(value: &nym_vpn_lib_types::GeoExclusionSettings) -> Self {
            Self {
                enabled: value.enabled,
                listen_port: value.listen_port,
                excluded_countries: value.excluded_countries.clone(),
            }
        }
    }
}
