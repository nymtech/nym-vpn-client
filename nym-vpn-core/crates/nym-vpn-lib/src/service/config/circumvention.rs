// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
pub(crate) mod v9 {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum FrontingMode {
        Off,
        #[default]
        OnRetry,
        Always,
    }

    impl From<nym_vpn_lib_types::FrontingMode> for FrontingMode {
        fn from(value: nym_vpn_lib_types::FrontingMode) -> Self {
            FrontingMode::from(&value)
        }
    }

    impl From<&nym_vpn_lib_types::FrontingMode> for FrontingMode {
        fn from(value: &nym_vpn_lib_types::FrontingMode) -> Self {
            match value {
                nym_vpn_lib_types::FrontingMode::Off => FrontingMode::Off,
                nym_vpn_lib_types::FrontingMode::OnRetry => FrontingMode::OnRetry,
                nym_vpn_lib_types::FrontingMode::Always => FrontingMode::Always,
            }
        }
    }

    impl From<FrontingMode> for nym_vpn_lib_types::FrontingMode {
        fn from(value: FrontingMode) -> Self {
            nym_vpn_lib_types::FrontingMode::from(&value)
        }
    }

    impl From<&FrontingMode> for nym_vpn_lib_types::FrontingMode {
        fn from(value: &FrontingMode) -> Self {
            match value {
                FrontingMode::Off => nym_vpn_lib_types::FrontingMode::Off,
                FrontingMode::OnRetry => nym_vpn_lib_types::FrontingMode::OnRetry,
                FrontingMode::Always => nym_vpn_lib_types::FrontingMode::Always,
            }
        }
    }
}
