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

        let network_stats = value
            .network_stats
            .ok_or(ConversionError::NoValueSet(
                "VpnServiceConfig.network_stats",
            ))?
            .into();

        let mt = value.mixnet_traffic.ok_or(ConversionError::NoValueSet(
            "VpnServiceConfig.mixnet_traffic",
        ))?;

        let mixnet_traffic = nym_vpn_lib_types::MixnetTrafficConfig {
            poisson_parameter: mt.poisson_parameter,
            average_packet_delay: mt.average_packet_delay,
            message_sending_average_delay: mt.message_sending_average_delay,
            disable_poisson_rate: mt.disable_poisson_rate,
            disable_background_cover_traffic: mt.disable_background_cover_traffic,
            min_mixnode_performance: mt.min_mixnode_performance.map(|u| u as u8),
            min_gateway_mixnet_performance: mt.min_gateway_mixnet_performance.map(|u| u as u8),
        };

        let config = nym_vpn_lib_types::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,

            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,
            network_stats,

            min_gateway_vpn_performance: value.min_gateway_vpn_performance.map(|u| u as u8),
            mixnet_traffic,
        };
        Ok(config)
    }
}

impl From<nym_vpn_lib_types::VpnServiceConfig> for proto::VpnServiceConfig {
    fn from(value: nym_vpn_lib_types::VpnServiceConfig) -> Self {
        let entry_point = Some(proto::EntryNode::from(value.entry_point));
        let exit_point = Some(proto::ExitNode::from(value.exit_point));
        let custom_dns = Some(proto::IpAddrList::from(value.custom_dns));
        let mixnet_traffic = Some(proto::MixnetTrafficConfig {
            poisson_parameter: value.mixnet_traffic.poisson_parameter,
            average_packet_delay: value.mixnet_traffic.average_packet_delay,
            message_sending_average_delay: value.mixnet_traffic.message_sending_average_delay,
            disable_poisson_rate: value.mixnet_traffic.disable_poisson_rate,
            disable_background_cover_traffic: value.mixnet_traffic.disable_background_cover_traffic,
            min_mixnode_performance: value
                .mixnet_traffic
                .min_mixnode_performance
                .map(|u| u as u32),
            min_gateway_mixnet_performance: value
                .mixnet_traffic
                .min_gateway_mixnet_performance
                .map(|u| u as u32),
        });
        proto::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,
            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,

            network_stats: Some(proto::NetworkStatsConfig::from(value.network_stats)),

            min_gateway_vpn_performance: value.min_gateway_vpn_performance.map(|u| u as u32),

            mixnet_traffic,
        }
    }
}
