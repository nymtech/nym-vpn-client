// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use time::OffsetDateTime;

use nym_vpn_lib_types_uniffi::{NymNetworkDetails, NymVpnNetwork};

uniffi::use_remote_type!(nym_vpn_lib_types_uniffi::OffsetDateTime);

#[derive(uniffi::Record)]
pub struct VpnServiceInfo {
    pub version: String,
    pub build_timestamp: Option<OffsetDateTime>,
    pub triple: String,
    pub platform: String,
    pub git_commit: String,
    pub nym_network: NymNetworkDetails,
    pub nym_vpn_network: NymVpnNetwork,
}

impl From<nym_vpnd_types::service::VpnServiceInfo> for VpnServiceInfo {
    fn from(info: nym_vpnd_types::service::VpnServiceInfo) -> Self {
        VpnServiceInfo {
            version: info.version,
            build_timestamp: info.build_timestamp,
            triple: info.triple,
            platform: info.platform,
            git_commit: info.git_commit,
            nym_network: NymNetworkDetails::from(info.nym_network.network),
            nym_vpn_network: NymVpnNetwork::from(info.nym_vpn_network),
        }
    }
}
