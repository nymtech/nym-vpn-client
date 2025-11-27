use anyhow::anyhow;
use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use ts_rs::TS;

use crate::country::Country;
use crate::state::app::VpnMode;

#[derive(Serialize, Deserialize, Debug, Clone, TS, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "tauri.ts")]
pub enum NodeConfig {
    Country(Country),
    /// Gateway ID
    Gateway(String),
    Region(String),
    Random,
}

impl NodeConfig {
    pub fn from_country_code(code: &str) -> anyhow::Result<Self> {
        Ok(NodeConfig::Country(
            Country::try_new_from_code(code)
                .ok_or_else(|| anyhow!("failed to create country from code '{}'", code))?,
        ))
    }
}

impl TryFrom<lib::EntryPoint> for NodeConfig {
    type Error = anyhow::Error;

    fn try_from(node: lib::EntryPoint) -> Result<Self, Self::Error> {
        Ok(match node {
            lib::EntryPoint::Country {
                two_letter_iso_country_code: code,
            } => NodeConfig::from_country_code(&code)?,
            lib::EntryPoint::Region { region } => NodeConfig::Region(region),
            lib::EntryPoint::Gateway { identity } => {
                NodeConfig::Gateway(identity.to_base58_string())
            }
            lib::EntryPoint::Random => NodeConfig::Random,
        })
    }
}

impl TryFrom<lib::ExitPoint> for NodeConfig {
    type Error = anyhow::Error;

    fn try_from(node: lib::ExitPoint) -> Result<Self, Self::Error> {
        Ok(match node {
            lib::ExitPoint::Country {
                two_letter_iso_country_code: code,
            } => NodeConfig::from_country_code(&code)?,
            lib::ExitPoint::Region { region } => NodeConfig::Region(region),
            lib::ExitPoint::Gateway { identity } => {
                NodeConfig::Gateway(identity.to_base58_string())
            }
            lib::ExitPoint::Random => NodeConfig::Random,
            lib::ExitPoint::Address { address: _ } => {
                // TODO add support for this type of exit point
                return Err(anyhow!(
                    "Exit node of type [Address] is not supported by tauri client"
                ));
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct VpndConfig {
    pub entry_node: NodeConfig,
    pub exit_node: NodeConfig,
    pub dns: Option<IpAddr>,
    pub allow_lan: bool,
    pub disable_ipv6: bool,
    pub vpn_mode: VpnMode,
    pub bridges: bool,
    pub netstack: bool,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub min_mixnode_performance: Option<u8>,
    pub min_gateway_mixnet_performance: Option<u8>,
    pub min_gateway_vpn_performance: Option<u8>,
    pub residential_exit: bool,
}

impl VpndConfig {
    pub fn from_lib(config: lib::VpnServiceConfig) -> anyhow::Result<Self> {
        let vpn_mode = if config.enable_two_hop {
            VpnMode::Mixnet
        } else {
            VpnMode::Wg
        };

        Ok(VpndConfig {
            entry_node: config.entry_point.try_into()?,
            exit_node: config.exit_point.try_into()?,
            dns: config.dns,
            allow_lan: config.allow_lan,
            disable_ipv6: config.disable_ipv6,
            vpn_mode,
            bridges: config.enable_bridges,
            netstack: config.netstack,
            disable_poisson_rate: config.disable_poisson_rate,
            disable_background_cover_traffic: config.disable_background_cover_traffic,
            min_mixnode_performance: config.min_mixnode_performance,
            min_gateway_mixnet_performance: config.min_gateway_mixnet_performance,
            min_gateway_vpn_performance: config.min_gateway_vpn_performance,
            residential_exit: config.residential_exit,
        })
    }
}
