// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

#[derive(uniffi::Record)]
pub struct VpnServiceInfo {
    pub version: String,
    pub build_timestamp: Option<i64>,
    pub triple: String,
    pub platform: String,
    pub git_commit: String,
    pub nym_network: String,
}

impl From<nym_vpnd_types::service::VpnServiceInfo> for VpnServiceInfo {
    fn from(info: nym_vpnd_types::service::VpnServiceInfo) -> Self {
        VpnServiceInfo {
            version: info.version,
            build_timestamp: info.build_timestamp.map(|ts| ts.unix_timestamp()),
            triple: info.triple,
            platform: info.platform,
            git_commit: info.git_commit,
            nym_network: info.nym_network.network.network_name,
        }
    }
}
