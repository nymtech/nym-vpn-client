// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{
    ConfigSetupError,
    config::{
        VpnServiceConfigExt,
        circumvention::v9::FrontingMode,
        entry_exit::v2::{EntryPoint, ExitPoint},
        gateway_selection_algorithm::v9::GatewaySelectionAlgorithmConfig,
        geo_exclusion_settings::v9::GeoExclusionSettings,
        mixnet_traffic::v5::MixnetTrafficConfig,
        network_stats::v1::NetworkStatisticsConfig,
        split_tunnel_settings::v8::SplitTunnelSettings,
    },
};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnServiceConfig {
    pub entry_point: EntryPoint,
    pub exit_point: ExitPoint,
    pub allow_lan: bool,
    pub disable_ipv6: bool,
    pub enable_two_hop: bool,
    pub enable_bridges: bool,
    pub enable_lewes_protocol: bool,
    pub enable_ad_blocking: bool,
    pub fronting_mode: FrontingMode,
    pub netstack: bool,
    pub min_gateway_vpn_performance: Option<u8>,
    pub residential_exit: bool,
    pub enable_custom_dns: bool,
    pub custom_dns: Vec<String>,
    pub mixnet_traffic: MixnetTrafficConfig,
    pub network_stats: NetworkStatisticsConfig,
    pub split_tunnel: SplitTunnelSettings,
    pub geo_exclusion: GeoExclusionSettings,
    pub gateway_selection_algorithm_config: GatewaySelectionAlgorithmConfig,
}

impl From<VpnServiceConfig> for VpnServiceConfigExt {
    fn from(v9: VpnServiceConfig) -> Self {
        VpnServiceConfigExt::V9(v9)
    }
}

impl TryFrom<VpnServiceConfig> for nym_vpn_lib_types::VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfig) -> Result<Self, Self::Error> {
        let entry_point = nym_vpn_lib_types::EntryPoint::try_from(value.entry_point)?;

        let exit_point = nym_vpn_lib_types::ExitPoint::try_from(value.exit_point)?;

        let custom_dns: Vec<IpAddr> = value
            .custom_dns
            .iter()
            .map(|dns_str| {
                IpAddr::from_str(dns_str)
                    .map_err(|e| ConfigSetupError::IpAddress { error: Box::new(e) })
            })
            .collect::<Result<_, _>>()?;

        let mixnet_traffic = nym_vpn_lib_types::MixnetTrafficConfig::from(value.mixnet_traffic);
        let network_stats = nym_vpn_lib_types::NetworkStatisticsConfig::from(value.network_stats);
        let split_tunnel = nym_vpn_lib_types::SplitTunnelSettings::from(value.split_tunnel);
        let geo_exclusion = nym_vpn_lib_types::GeoExclusionSettings::try_from(value.geo_exclusion)
            .unwrap_or_else(|error| {
                tracing::warn!(
                    "Invalid persisted geo-exclusion settings, resetting to default: {error}"
                );
                nym_vpn_lib_types::GeoExclusionSettings::default()
            });
        let gateway_selection_algorithm_config =
            nym_vpn_lib_types::GatewaySelectionAlgorithmConfig::from(
                value.gateway_selection_algorithm_config,
            );
        let fronting_mode = nym_vpn_lib_types::FrontingMode::from(value.fronting_mode);

        Ok(nym_vpn_lib_types::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            enable_ad_blocking: value.enable_ad_blocking,
            fronting_mode,
            netstack: value.netstack,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,
            mixnet_traffic,
            network_stats,
            split_tunnel,
            geo_exclusion,
            gateway_selection_algorithm_config,
            ..Default::default()
        })
    }
}
