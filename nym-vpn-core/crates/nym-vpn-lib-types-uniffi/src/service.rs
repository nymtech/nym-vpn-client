use crate::{EntryPoint, ExitPoint};

#[derive(uniffi::Record)]
pub struct VpnServiceConfig {
    pub entry_point: EntryPoint,
    pub exit_point: ExitPoint,
    pub dns: Option<String>,
    pub allow_lan: bool,
    pub disable_ipv6: bool,
    pub enable_two_hop: bool,
    pub enable_bridges: bool,
    pub netstack: bool,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub min_mixnode_performance: Option<u8>,
    pub min_gateway_mixnet_performance: Option<u8>,
    pub min_gateway_vpn_performance: Option<u8>,
    pub residential_exit: bool,
}

impl From<nym_vpn_lib_types::service::VpnServiceConfig> for VpnServiceConfig {
    fn from(value: nym_vpn_lib_types::service::VpnServiceConfig) -> Self {
        Self {
            entry_point: value.entry_point.into(),
            exit_point: value.exit_point.into(),
            dns: value.dns.map(|ip| ip.to_string()),
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance,
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
            residential_exit: value.residential_exit,
        }
    }
}

impl From<VpnServiceConfig> for nym_vpn_lib_types::service::VpnServiceConfig {
    fn from(value: VpnServiceConfig) -> Self {
        Self {
            entry_point: value.entry_point.into(),
            exit_point: value.exit_point.into(),
            dns: value.dns.and_then(|ip| ip.parse().ok()),
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance,
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
            residential_exit: value.residential_exit,
        }
    }
}

pub type BoxedVpnServiceConfig = Box<VpnServiceConfig>;

uniffi::custom_type!(BoxedVpnServiceConfig, VpnServiceConfig, {
    remote,
    try_lift: |val| Ok(Box::new(val)),
    lower: |val| *val
});
