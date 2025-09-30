// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(uniffi::Record, Clone, PartialEq)]
pub struct NetworkCompatibility {
    pub core: String,
    pub ios: String,
    pub macos: String,
    pub tauri: String,
    pub android: String,
}

impl From<nym_vpn_lib_types::NetworkCompatibility> for NetworkCompatibility {
    fn from(value: nym_vpn_lib_types::NetworkCompatibility) -> Self {
        NetworkCompatibility {
            core: value.core,
            ios: value.ios,
            macos: value.macos,
            tauri: value.tauri,
            android: value.android,
        }
    }
}

impl From<nym_vpn_api_client::NetworkCompatibility> for NetworkCompatibility {
    fn from(value: nym_vpn_api_client::NetworkCompatibility) -> Self {
        NetworkCompatibility {
            core: value.core,
            ios: value.ios,
            macos: value.macos,
            tauri: value.tauri,
            android: value.android,
        }
    }
}
