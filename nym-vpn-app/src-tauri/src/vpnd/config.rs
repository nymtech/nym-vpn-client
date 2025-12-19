use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use ts_rs::TS;

use crate::state::app::VpnMode;
use crate::vpnd::node::Node;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct VpndConfig {
    pub entry_node: Node,
    pub exit_node: Node,
    pub custom_dns: Option<Vec<IpAddr>>,
    pub enable_custom_dns: bool,
    pub allow_lan: bool,
    pub disable_ipv6: bool,
    pub vpn_mode: VpnMode,
    pub bridges: bool,
    pub netstack: bool,
    pub min_gateway_vpn_performance: Option<u8>,
    pub residential_exit: bool,
    //pub mixnet_traffic: lib::MixnetTrafficConfig,
}

impl VpndConfig {
    pub fn from_lib(config: lib::VpnServiceConfig) -> anyhow::Result<Self> {
        let vpn_mode = if config.enable_two_hop {
            VpnMode::Wg
        } else {
            VpnMode::Mixnet
        };

        Ok(VpndConfig {
            entry_node: config.entry_point.into(),
            exit_node: config.exit_point.try_into()?,
            custom_dns: Some(config.custom_dns),
            enable_custom_dns: config.enable_custom_dns,
            allow_lan: config.allow_lan,
            disable_ipv6: config.disable_ipv6,
            vpn_mode,
            bridges: config.enable_bridges,
            netstack: config.netstack,
            min_gateway_vpn_performance: config.min_gateway_vpn_performance,
            residential_exit: config.residential_exit,
            //mixnet_traffic: config.mixnet_traffic,
        })
    }
}
