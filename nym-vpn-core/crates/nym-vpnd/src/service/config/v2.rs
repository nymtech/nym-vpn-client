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
pub struct VpnServiceConfig {
    entry_point: EntryPoint,
    exit_point: ExitPoint,
    dns: Option<String>,
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
}

impl From<VpnServiceConfig> for VpnServiceConfigExt {
    fn from(v2: VpnServiceConfig) -> Self {
        VpnServiceConfigExt::V2(v2)
    }
}

impl TryFrom<VpnServiceConfig> for nym_vpn_lib_types::VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfig) -> Result<Self, Self::Error> {
        let custom_dns = value
            .dns
            .map(|addr| {
                IpAddr::from_str(&addr)
                    .map(|ip| vec![ip])
                    .map_err(|e| ConfigSetupError::IpAddress { error: Box::new(e) })
            })
            .transpose()?;

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
            custom_dns,
            network_stats: Default::default(),
        };
        Ok(config)
    }
}
