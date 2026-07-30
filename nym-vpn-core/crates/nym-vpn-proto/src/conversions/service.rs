// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{conversions::ConversionError, proto};
use std::net::IpAddr;

impl TryFrom<proto::VpnServiceConfig> for nym_vpn_lib_types::VpnServiceConfig {
    type Error = ConversionError;

    fn try_from(value: proto::VpnServiceConfig) -> Result<Self, Self::Error> {
        let entry_point = value
            .entry_point
            .map(nym_vpn_lib_types::EntryPoint::try_from)
            .transpose()?
            .ok_or(ConversionError::NoValueSet("VpnServiceConfig.entry_point"))?;

        let exit_point = value
            .exit_point
            .map(nym_vpn_lib_types::ExitPoint::try_from)
            .transpose()?
            .ok_or(ConversionError::NoValueSet("VpnServiceConfig.exit_point"))?;

        let custom_dns: Vec<IpAddr> = match value.custom_dns {
            Some(ip_addr_list) => ip_addr_list.try_into()?,
            None => vec![],
        };

        let mixnet_traffic = value
            .mixnet_traffic
            .map(nym_vpn_lib_types::MixnetTrafficConfig::from)
            .ok_or(ConversionError::NoValueSet(
                "VpnServiceConfig.mixnet_traffic",
            ))?;

        let network_stats = value
            .network_stats
            .map(nym_vpn_lib_types::NetworkStatisticsConfig::from)
            .ok_or(ConversionError::NoValueSet(
                "VpnServiceConfig.network_stats",
            ))?;

        let split_tunnel = value
            .split_tunnel
            .map(nym_vpn_lib_types::SplitTunnelSettings::from)
            .ok_or(ConversionError::NoValueSet("VpnServiceConfig.split_tunnel"))?;

        let geo_exclusion = value
            .geo_exclusion
            .map(nym_vpn_lib_types::GeoExclusionSettings::from)
            .ok_or(ConversionError::NoValueSet(
                "VpnServiceConfig.geo_exclusion",
            ))?;

        let gateway_selection_algorithm_config = value
            .gateway_selection_algorithm_config
            .map(nym_vpn_lib_types::GatewaySelectionAlgorithmConfig::from)
            .ok_or(ConversionError::NoValueSet(
                "VpnServiceConfig.gateway_selection_algorithm",
            ))?;
        let fronting_mode = proto::FrontingModes::try_from(value.fronting_mode)
            .map_err(|e| ConversionError::Decode("fronting_mode", e))?
            .into();
        let gateway_independence = value
            .gateway_independence
            .map(nym_vpn_lib_types::GatewayIndependence::from)
            .ok_or(ConversionError::NoValueSet(
                "VpnServiceConfig.gateway_independence",
            ))?;

        let config = nym_vpn_lib_types::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            enable_ad_blocking: value.enable_ad_blocking,
            fronting_mode,
            netstack: value.netstack,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance.map(|u| u as u8),
            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,
            mixnet_traffic,
            network_stats,
            split_tunnel,
            geo_exclusion,
            gateway_selection_algorithm_config,
            gateway_independence,
        };
        Ok(config)
    }
}

impl From<nym_vpn_lib_types::VpnServiceConfig> for proto::VpnServiceConfig {
    fn from(value: nym_vpn_lib_types::VpnServiceConfig) -> Self {
        let entry_point = Some(proto::EntryNode::from(value.entry_point));
        let exit_point = Some(proto::ExitNode::from(value.exit_point));
        let custom_dns = Some(proto::IpAddrList::from(value.custom_dns));
        let mixnet_traffic = Some(proto::MixnetTrafficConfig::from(value.mixnet_traffic));
        let network_stats = Some(proto::NetworkStatsConfig::from(value.network_stats));
        let split_tunnel = Some(proto::SplitTunnelSettings::from(value.split_tunnel));
        let geo_exclusion = Some(proto::GeoExclusionSettings::from(value.geo_exclusion));
        let gateway_selection_algorithm_config =
            proto::GatewaySelectionAlgorithmConfig::from(value.gateway_selection_algorithm_config)
                .into();
        let gateway_independence =
            Some(proto::GatewayIndependence::from(value.gateway_independence));

        proto::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            enable_ad_blocking: value.enable_ad_blocking,
            fronting_mode: proto::FrontingModes::from(value.fronting_mode).into(),
            netstack: value.netstack,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance.map(|u| u as u32),
            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,
            mixnet_traffic,
            network_stats,
            split_tunnel,
            geo_exclusion,
            gateway_selection_algorithm_config,
            gateway_independence,
        }
    }
}

impl From<proto::MixnetTrafficConfig> for nym_vpn_lib_types::MixnetTrafficConfig {
    fn from(value: proto::MixnetTrafficConfig) -> Self {
        nym_vpn_lib_types::MixnetTrafficConfig {
            poisson_parameter_for_loop_cover_stream: value.poisson_parameter_for_loop_cover_stream,
            average_packet_delay: value.average_packet_delay,
            message_sending_average_delay: value.message_sending_average_delay,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance.map(|u| u as u8),
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance.map(|u| u as u8),
        }
    }
}

impl From<proto::GeoExclusionSettings> for nym_vpn_lib_types::GeoExclusionSettings {
    fn from(value: proto::GeoExclusionSettings) -> Self {
        nym_vpn_lib_types::GeoExclusionSettings {
            enabled: value.enabled,
            listen_port: value.listen_port as u16,
            excluded_countries: value.excluded_countries,
        }
    }
}

impl From<nym_vpn_lib_types::GeoExclusionSettings> for proto::GeoExclusionSettings {
    fn from(value: nym_vpn_lib_types::GeoExclusionSettings) -> Self {
        proto::GeoExclusionSettings {
            enabled: value.enabled,
            listen_port: value.listen_port as u32,
            excluded_countries: value.excluded_countries,
        }
    }
}

impl From<nym_vpn_lib_types::MixnetTrafficConfig> for proto::MixnetTrafficConfig {
    fn from(value: nym_vpn_lib_types::MixnetTrafficConfig) -> Self {
        proto::MixnetTrafficConfig {
            poisson_parameter_for_loop_cover_stream: value.poisson_parameter_for_loop_cover_stream,
            average_packet_delay: value.average_packet_delay,
            message_sending_average_delay: value.message_sending_average_delay,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance.map(|u| u as u32),
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance.map(|u| u as u32),
        }
    }
}

impl From<nym_vpn_lib_types::FrontingMode> for proto::FrontingModes {
    fn from(value: nym_vpn_lib_types::FrontingMode) -> Self {
        match value {
            nym_vpn_lib_types::FrontingMode::Off => proto::FrontingModes::Off,
            nym_vpn_lib_types::FrontingMode::OnRetry => proto::FrontingModes::OnRetry,
            nym_vpn_lib_types::FrontingMode::Always => proto::FrontingModes::Always,
        }
    }
}

impl From<proto::FrontingModes> for nym_vpn_lib_types::FrontingMode {
    fn from(val: proto::FrontingModes) -> Self {
        match val {
            proto::FrontingModes::Off => nym_vpn_lib_types::FrontingMode::Off,
            proto::FrontingModes::OnRetry => nym_vpn_lib_types::FrontingMode::OnRetry,
            proto::FrontingModes::Always => nym_vpn_lib_types::FrontingMode::Always,
        }
    }
}
