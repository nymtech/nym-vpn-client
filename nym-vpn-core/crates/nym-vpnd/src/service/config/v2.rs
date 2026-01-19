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

impl From<VpnServiceConfig> for VpnServiceConfigExt {
    fn from(v2: VpnServiceConfig) -> Self {
        VpnServiceConfigExt::V2(v2)
    }
}

impl TryFrom<VpnServiceConfig> for nym_vpn_lib_types::VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfig) -> Result<Self, Self::Error> {
        let entry_point = nym_vpn_lib_types::EntryPoint::try_from(value.entry_point)?;

        let exit_point = nym_vpn_lib_types::ExitPoint::try_from(value.exit_point)?;

        let custom_dns = match value.dns {
            Some(str) => {
                let ip = IpAddr::from_str(&str)
                    .map_err(|e| ConfigSetupError::IpAddress { error: Box::new(e) })?;
                vec![ip]
            }
            None => vec![],
        };

        let mixnet_traffic = nym_vpn_lib_types::MixnetTrafficConfig {
            poisson_parameter_for_loop_cover_stream: None,
            average_packet_delay: None,
            message_sending_average_delay: None,

            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,

            min_mixnode_performance: value.min_mixnode_performance,
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance,
        };

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
            enable_custom_dns: !custom_dns.is_empty(),
            custom_dns,
            mixnet_traffic,
            ..Default::default()
        };
        Ok(config)
    }
}
