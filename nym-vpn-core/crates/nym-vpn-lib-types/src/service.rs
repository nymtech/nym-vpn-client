// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, net::IpAddr};

use nym_gateway_directory::{EntryPoint, ExitPoint};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VpnServiceConfig {
    pub entry_point: EntryPoint,
    pub exit_point: ExitPoint,
    pub dns: Option<IpAddr>,
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

impl fmt::Display for VpnServiceConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "entry point: {}, exit point: {}, dns: {}",
            self.entry_point,
            self.exit_point,
            self.dns
                .map(|d| d.to_string())
                .unwrap_or_else(|| "<None>".to_string())
        )?;
        writeln!(
            f,
            "allow_lan: {}, disable_ipv6: {}, enable_two_hop: {}, netstack: {}",
            self.allow_lan, self.disable_ipv6, self.enable_two_hop, self.netstack
        )?;
        writeln!(
            f,
            "disable_poisson_rate: {}, disable_background_cover_traffic: {}",
            self.disable_poisson_rate, self.disable_background_cover_traffic
        )?;
        writeln!(
            f,
            "min_mixnode_performance: {}, min_gateway_mixnet_performance: {}, min_gateway_vpn_performance: {}",
            self.min_mixnode_performance
                .map(|p| p.to_string())
                .unwrap_or_else(|| "<None>".to_string()),
            self.min_gateway_mixnet_performance
                .map(|p| p.to_string())
                .unwrap_or_else(|| "<None>".to_string()),
            self.min_gateway_vpn_performance
                .map(|p| p.to_string())
                .unwrap_or_else(|| "<None>".to_string())
        )?;
        writeln!(f, "residential_exit: {}", self.residential_exit)?;
        Ok(())
    }
}

impl Default for VpnServiceConfig {
    fn default() -> Self {
        Self {
            entry_point: EntryPoint::Random,
            exit_point: ExitPoint::Random,
            dns: None,
            allow_lan: false,
            disable_ipv6: false,
            enable_two_hop: false,
            enable_bridges: false,
            netstack: false,
            disable_poisson_rate: false,
            disable_background_cover_traffic: false,
            min_mixnode_performance: None,
            min_gateway_mixnet_performance: None,
            min_gateway_vpn_performance: None,
            residential_exit: false,
        }
    }
}
