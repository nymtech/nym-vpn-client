// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{
    ConfigSetupError,
    config::{
        VpnServiceConfigExt,
        entry_exit::v2::{EntryPoint, ExitPoint},
    },
};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct VpnServiceConfig {
    pub(crate) entry_point: EntryPoint,
    pub(crate) exit_point: ExitPoint,
    pub(crate) allow_lan: bool,
    pub(crate) disable_ipv6: bool,
    pub(crate) enable_two_hop: bool,
    pub(crate) enable_bridges: bool,
    pub(crate) netstack: bool,
    pub(crate) disable_poisson_rate: bool,
    pub(crate) disable_background_cover_traffic: bool,
    pub(crate) min_mixnode_performance: Option<u8>,
    pub(crate) min_gateway_mixnet_performance: Option<u8>,
    pub(crate) min_gateway_vpn_performance: Option<u8>,
    pub(crate) residential_exit: bool,
    pub(crate) custom_dns: Option<Vec<String>>,
}

impl From<VpnServiceConfig> for VpnServiceConfigExt {
    fn from(v3: VpnServiceConfig) -> Self {
        VpnServiceConfigExt::V3(v3)
    }
}

impl TryFrom<VpnServiceConfig> for nym_vpn_lib_types::VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfig) -> Result<Self, Self::Error> {
        let custom_dns = match &value.custom_dns {
            None => vec![],
            Some(dns_list) if dns_list.is_empty() => vec![],
            Some(dns_list) => dns_list
                .iter()
                .map(|dns_str| {
                    IpAddr::from_str(dns_str)
                        .map_err(|e| ConfigSetupError::IpAddress { error: Box::new(e) })
                })
                .collect::<Result<_, _>>()?,
        };

        let config = nym_vpn_lib_types::VpnServiceConfig {
            entry_point: nym_vpn_lib_types::EntryPoint::try_from(value.entry_point)?,
            exit_point: nym_vpn_lib_types::ExitPoint::try_from(value.exit_point)?,
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
            enable_custom_dns: !custom_dns.is_empty(),
            custom_dns,
        };
        Ok(config)
    }
}
