// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{
    ConfigSetupError,
    config::{
        VpnServiceConfigExt, VpnServiceConfigExtLatest,
        v2::{EntryPointExtV2, ExitPointExtV2},
    },
    error::Result,
};
use nym_vpn_lib_types::{EntryPoint, ExitPoint, VpnServiceConfig};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnServiceConfigExtV3 {
    entry_point: EntryPointExtV2,
    exit_point: ExitPointExtV2,
    allow_lan: bool,
    disable_ipv6: bool,
    enable_two_hop: bool,
    enable_bridges: bool,
    netstack: bool,
    disable_poisson_rate: bool,
    disable_background_cover_traffic: bool,
    min_mixnode_performance: Option<u8>,
    min_gateway_mixnet_performance: Option<u8>,
    min_gateway_vpn_performance: Option<u8>,
    residential_exit: bool,
    custom_dns: Option<Vec<String>>,
}

impl From<VpnServiceConfigExtV3> for VpnServiceConfigExt {
    fn from(v3: VpnServiceConfigExtV3) -> Self {
        VpnServiceConfigExt::V3(v3)
    }
}

impl TryFrom<VpnServiceConfigExtV3> for VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfigExtV3) -> Result<Self, Self::Error> {
        let custom_dns: Option<Vec<IpAddr>> = value
            .custom_dns
            .map(|dns_list| {
                dns_list
                    .into_iter()
                    .map(|addr| {
                        IpAddr::from_str(&addr)
                            .map_err(|e| ConfigSetupError::IpAddress { error: Box::new(e) })
                    })
                    .collect::<Result<Vec<IpAddr>, ConfigSetupError>>()
            })
            .transpose()?;

        let config = VpnServiceConfig {
            entry_point: EntryPoint::try_from(value.entry_point)?,
            exit_point: ExitPoint::try_from(value.exit_point)?,
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
            custom_dns,
        };
        Ok(config)
    }
}

//
// Note: impl TryFrom<&VpnServiceConfig> is only required for the latest configuration version,
// so when the next version is created, MOVE this impl to that version.
//
impl TryFrom<&VpnServiceConfig> for VpnServiceConfigExtLatest {
    type Error = ConfigSetupError;

    fn try_from(value: &VpnServiceConfig) -> Result<Self, Self::Error> {
        let custom_dns = match &value.custom_dns {
            None => None,
            Some(dns_list) => {
                let string_list: Vec<String> =
                    dns_list.iter().map(|addr| addr.to_string()).collect();
                Some(string_list)
            }
        };
        let ext_config = VpnServiceConfigExtLatest {
            entry_point: EntryPointExtV2::try_from(&value.entry_point)?,
            exit_point: ExitPointExtV2::try_from(&value.exit_point)?,
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
            custom_dns,
        };
        Ok(ext_config)
    }
}
