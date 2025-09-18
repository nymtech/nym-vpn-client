// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

use super::{EntryPointExtV1, ExitPointExtV1, VpnServiceConfigExtV1};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnServiceConfigExtV2 {
    pub entry_point: EntryPointExtV1,
    pub exit_point: ExitPointExtV1,
    pub dns: Option<String>,
    pub disable_ipv6: bool,
    pub enable_two_hop: bool,
    pub netstack: bool,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub min_mixnode_performance: Option<u8>,
    pub min_gateway_mixnet_performance: Option<u8>,
    pub min_gateway_vpn_performance: Option<u8>,
}

impl Default for VpnServiceConfigExtV2 {
    fn default() -> Self {
        Self {
            entry_point: EntryPointExtV1::Random,
            exit_point: ExitPointExtV1::Random,
            dns: None,
            disable_ipv6: false,
            enable_two_hop: false,
            netstack: false,
            disable_poisson_rate: false,
            disable_background_cover_traffic: false,
            min_mixnode_performance: None,
            min_gateway_mixnet_performance: None,
            min_gateway_vpn_performance: None,
        }
    }
}

impl From<VpnServiceConfigExtV1> for VpnServiceConfigExtV2 {
    fn from(value: VpnServiceConfigExtV1) -> Self {
        Self {
            entry_point: value.entry_point,
            exit_point: value.exit_point,
            ..Default::default()
        }
    }
}
