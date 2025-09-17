// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use time::OffsetDateTime;

use nym_vpn_lib_types_uniffi::{EntryPoint, ExitPoint, NymNetworkDetails, NymVpnNetwork};

#[derive(uniffi::Record)]
pub struct VpnServiceConfig {
    pub entry_point: EntryPoint,
    pub exit_point: ExitPoint,
    pub dns: Option<String>,
    pub disable_ipv6: bool,
    pub enable_two_hop: bool,
    pub netstack: bool,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub min_mixnode_performance: Option<u8>,
    pub min_gateway_mixnet_performance: Option<u8>,
    pub min_gateway_vpn_performance: Option<u8>,
}

impl From<nym_vpnd_types::service::VpnServiceConfig> for VpnServiceConfig {
    fn from(config: nym_vpnd_types::service::VpnServiceConfig) -> Self {
        VpnServiceConfig {
            entry_point: EntryPoint::from(config.entry_point),
            exit_point: ExitPoint::from(config.exit_point),
            dns: config.dns.map(|ip| ip.to_string()),
            disable_ipv6: config.disable_ipv6,
            enable_two_hop: config.enable_two_hop,
            netstack: config.netstack,
            disable_poisson_rate: config.disable_poisson_rate,
            disable_background_cover_traffic: config.disable_background_cover_traffic,
            min_mixnode_performance: config.min_mixnode_performance,
            min_gateway_mixnet_performance: config.min_gateway_mixnet_performance,
            min_gateway_vpn_performance: config.min_gateway_vpn_performance,
        }
    }
}

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
