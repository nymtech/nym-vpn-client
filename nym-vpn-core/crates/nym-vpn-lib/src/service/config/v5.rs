// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{
    ConfigSetupError,
    config::{
        VpnServiceConfigExt,
        entry_exit::v2::{EntryPoint, ExitPoint},
        mixnet_traffic::v5::MixnetTrafficConfig,
        network_stats::v1::NetworkStatisticsConfig,
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
    pub netstack: bool,
    pub min_gateway_vpn_performance: Option<u8>,
    pub residential_exit: bool,
    pub enable_custom_dns: bool,
    pub custom_dns: Vec<String>,
    pub mixnet_traffic: MixnetTrafficConfig,
    pub network_stats: NetworkStatisticsConfig,
}

impl From<VpnServiceConfig> for VpnServiceConfigExt {
    fn from(v5: VpnServiceConfig) -> Self {
        VpnServiceConfigExt::V5(v5)
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

        let config = nym_vpn_lib_types::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,
            mixnet_traffic,
            network_stats,
            ..Default::default()
        };

        Ok(config)
    }
}
