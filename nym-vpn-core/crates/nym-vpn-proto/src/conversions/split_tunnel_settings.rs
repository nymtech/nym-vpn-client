// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::proto;

impl From<nym_vpn_lib_types::SplitApp> for proto::SplitApp {
    fn from(value: nym_vpn_lib_types::SplitApp) -> Self {
        Self { path: value.path }
    }
}

impl From<proto::SplitApp> for nym_vpn_lib_types::SplitApp {
    fn from(value: proto::SplitApp) -> Self {
        Self { path: value.path }
    }
}

impl From<nym_vpn_lib_types::SplitTunnelSettings> for proto::SplitTunnelSettings {
    fn from(value: nym_vpn_lib_types::SplitTunnelSettings) -> Self {
        Self {
            enabled: value.enabled,
            apps: value.apps.into_iter().map(proto::SplitApp::from).collect(),
        }
    }
}

impl From<proto::SplitTunnelSettings> for nym_vpn_lib_types::SplitTunnelSettings {
    fn from(value: proto::SplitTunnelSettings) -> Self {
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
