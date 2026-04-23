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

        let airporting = value
            .airporting
            .map(nym_vpn_lib_types::AirportingSettings::from)
            .ok_or(ConversionError::NoValueSet("VpnServiceConfig.airporting"))?;

        let config = nym_vpn_lib_types::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            enable_lewes_protocol: value.enable_lewes_protocol,
            enable_ad_blocking: value.enable_ad_blocking,
            netstack: value.netstack,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance.map(|u| u as u8),
            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,
            mixnet_traffic,
            network_stats,
            split_tunnel,
            airporting,
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
        let airporting = Some(proto::AirportingSettings::from(value.airporting));

        proto::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            enable_lewes_protocol: value.enable_lewes_protocol,
            enable_ad_blocking: value.enable_ad_blocking,
            netstack: value.netstack,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance.map(|u| u as u32),
            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,
            mixnet_traffic,
            network_stats,
            split_tunnel,
            airporting,
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

impl From<proto::AirportingSettings> for nym_vpn_lib_types::AirportingSettings {
    fn from(value: proto::AirportingSettings) -> Self {
        nym_vpn_lib_types::AirportingSettings {
            enabled: value.enabled,
            listen_port: value.listen_port as u16,
            excluded_countries: value.excluded_countries,
        }
    }
}

impl From<nym_vpn_lib_types::AirportingSettings> for proto::AirportingSettings {
    fn from(value: nym_vpn_lib_types::AirportingSettings) -> Self {
        proto::AirportingSettings {
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
