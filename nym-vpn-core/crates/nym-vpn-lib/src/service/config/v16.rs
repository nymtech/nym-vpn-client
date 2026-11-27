// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{
    ConfigSetupError,
    config::{
        VpnServiceConfigExt,
        circumvention::v9::FrontingMode,
        custom_dns::v15::CustomDnsSettings,
        default_dns::v16::DefaultDnsSettings,
        entry_exit::v3::{EntryPoint, ExitPoint},
        gateway_independence::v11::GatewayIndependence,
        gateway_selection_algorithm::v12::GatewaySelectionAlgorithmConfig,
        geo_exclusion_settings::v9::GeoExclusionSettings,
        mixnet_traffic::v5::MixnetTrafficConfig,
        network_stats::v1::NetworkStatisticsConfig,
        split_tunnel_settings::v8::SplitTunnelSettings,
        v14::BridgeMode,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeTransport {
    Quic,
    Tls,
    Ssh,
}

impl From<BridgeTransport> for nym_vpn_lib_types::BridgeTransport {
    fn from(value: BridgeTransport) -> Self {
        match value {
            BridgeTransport::Quic => Self::Quic,
            BridgeTransport::Tls => Self::Tls,
            BridgeTransport::Ssh => Self::Ssh,
        }
    }
}

impl From<nym_vpn_lib_types::BridgeTransport> for BridgeTransport {
    fn from(value: nym_vpn_lib_types::BridgeTransport) -> Self {
        match value {
            nym_vpn_lib_types::BridgeTransport::Quic => Self::Quic,
            nym_vpn_lib_types::BridgeTransport::Tls => Self::Tls,
            nym_vpn_lib_types::BridgeTransport::Ssh => Self::Ssh,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnServiceConfig {
    pub entry_point: EntryPoint,
    pub exit_point: ExitPoint,
    pub allow_lan: bool,
    pub disable_ipv6: bool,
    pub enable_two_hop: bool,
    pub bridge_mode: BridgeMode,
    pub enable_ad_blocking: bool,
    pub enable_conflict_detection: bool,
    pub fronting_mode: FrontingMode,
    pub transport_preference: Option<BridgeTransport>,
    pub netstack: bool,
    pub min_gateway_vpn_performance: Option<u8>,
    pub residential_exit: bool,
    pub custom_dns: CustomDnsSettings,
    #[serde(default)]
    pub default_dns: DefaultDnsSettings,
    pub mixnet_traffic: MixnetTrafficConfig,
    pub network_stats: NetworkStatisticsConfig,
    pub split_tunnel: SplitTunnelSettings,
    pub geo_exclusion: GeoExclusionSettings,
    pub gateway_selection_algorithm_config: GatewaySelectionAlgorithmConfig,
    pub gateway_independence: GatewayIndependence,
    #[serde(default)]
    pub auto_connect: bool,
}

impl From<VpnServiceConfig> for VpnServiceConfigExt {
    fn from(v16: VpnServiceConfig) -> Self {
        VpnServiceConfigExt::V16(v16)
    }
}

impl TryFrom<VpnServiceConfig> for nym_vpn_lib_types::VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfig) -> Result<Self, Self::Error> {
        let entry_point = nym_vpn_lib_types::EntryPoint::try_from(value.entry_point)?;

        let exit_point = nym_vpn_lib_types::ExitPoint::try_from(value.exit_point)?;

        let custom_dns = nym_vpn_lib_types::CustomDnsSettings::from(value.custom_dns);
        let default_dns = nym_vpn_lib_types::DefaultDnsSettings::from(value.default_dns);
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
        let gateway_independence =
            nym_vpn_lib_types::GatewayIndependence::from(value.gateway_independence);

        Ok(nym_vpn_lib_types::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            bridge_mode: value.bridge_mode.into(),
            enable_ad_blocking: value.enable_ad_blocking,
            enable_conflict_detection: value.enable_conflict_detection,
            fronting_mode,
            transport_preference: value.transport_preference.map(Into::into),
            netstack: value.netstack,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
            residential_exit: value.residential_exit,
            custom_dns,
            default_dns,
            mixnet_traffic,
            network_stats,
            split_tunnel,
            geo_exclusion,
            gateway_selection_algorithm_config,
            gateway_independence,
            auto_connect: value.auto_connect,
        })
    }
}
