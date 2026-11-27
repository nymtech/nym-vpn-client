// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) mod v16 {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct DefaultDnsSettings {
        #[serde(default = "default_enable_quad9")]
        pub enable_quad9: bool,
    }

    impl Default for DefaultDnsSettings {
        fn default() -> Self {
            Self {
                enable_quad9: default_enable_quad9(),
            }
        }
    }

    fn default_enable_quad9() -> bool {
        true
    }

    impl From<DefaultDnsSettings> for nym_vpn_lib_types::DefaultDnsSettings {
        fn from(value: DefaultDnsSettings) -> Self {
            Self {
                enable_quad9: value.enable_quad9,
            }
        }
    }

    // This is only required for the latest version of the external representation
    impl From<&nym_vpn_lib_types::DefaultDnsSettings> for DefaultDnsSettings {
        fn from(value: &nym_vpn_lib_types::DefaultDnsSettings) -> Self {
            Self {
                enable_quad9: value.enable_quad9,
            }
        }
    }
}
