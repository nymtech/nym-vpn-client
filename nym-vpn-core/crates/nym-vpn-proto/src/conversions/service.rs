use crate::{conversions::ConversionError, proto};

impl TryFrom<proto::VpnServiceConfig> for nym_vpn_lib_types::VpnServiceConfig {
    type Error = ConversionError;

    fn try_from(value: proto::VpnServiceConfig) -> Result<Self, Self::Error> {
        let entry_point = value
            .entry_point
            .map(nym_gateway_directory::EntryPoint::try_from)
            .transpose()?
            .ok_or(ConversionError::NoValueSet("VpnServiceConfig.entry_point"))?;

        let exit_point = value
            .exit_point
            .map(nym_gateway_directory::ExitPoint::try_from)
            .transpose()?
            .ok_or(ConversionError::NoValueSet("VpnServiceConfig.exit_point"))?;

        let dns = match value.dns.and_then(|dns| dns.ip) {
            Some(ip) => Some(
                ip.parse::<std::net::IpAddr>()
                    .map_err(|e| ConversionError::ParseAddr("VpnServiceConfig.dns", e))?,
            ),
            None => None,
        };

        let config = nym_vpn_lib_types::VpnServiceConfig {
            entry_point,
            exit_point,
            dns,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance.map(|u| u as u8),
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance.map(|u| u as u8),
            min_gateway_vpn_performance: value.min_gateway_vpn_performance.map(|u| u as u8),
        };
        Ok(config)
    }
}

impl From<nym_vpn_lib_types::VpnServiceConfig> for proto::VpnServiceConfig {
    fn from(value: nym_vpn_lib_types::VpnServiceConfig) -> Self {
        proto::VpnServiceConfig {
            entry_point: Some(proto::EntryNode::from(value.entry_point)),
            exit_point: Some(proto::ExitNode::from(value.exit_point)),
            dns: value.dns.map(|ip| proto::Dns {
                ip: Some(ip.to_string()),
            }),
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance.map(|u| u as u32),
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance.map(|u| u as u32),
            min_gateway_vpn_performance: value.min_gateway_vpn_performance.map(|u| u as u32),
        }
    }
}
