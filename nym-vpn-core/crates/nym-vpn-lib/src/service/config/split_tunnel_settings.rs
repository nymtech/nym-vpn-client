// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod v8 {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct SplitApp {
        pub path: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct SplitTunnelSettings {
        pub enabled: bool,
        pub apps: Vec<SplitApp>,
    }

    impl From<SplitApp> for nym_vpn_lib_types::SplitApp {
        fn from(value: SplitApp) -> Self {
            Self { path: value.path }
        }
    }

    impl From<nym_vpn_lib_types::SplitApp> for SplitApp {
        fn from(value: nym_vpn_lib_types::SplitApp) -> Self {
            Self { path: value.path }
        }
    }

    impl From<SplitTunnelSettings> for nym_vpn_lib_types::SplitTunnelSettings {
        fn from(value: SplitTunnelSettings) -> Self {
            Self {
                enabled: value.enabled,
                apps: value
                    .apps
                    .into_iter()
                    .map(nym_vpn_lib_types::SplitApp::from)
                    .collect(),
            }
        }
    }

    impl From<&nym_vpn_lib_types::SplitTunnelSettings> for SplitTunnelSettings {
        fn from(value: &nym_vpn_lib_types::SplitTunnelSettings) -> Self {
            Self {
                enabled: value.enabled,
                apps: value.apps.iter().cloned().map(SplitApp::from).collect(),
            }
        }
    }
}
