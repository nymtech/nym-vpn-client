use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use ts_rs::TS;

use crate::state::app::VpnMode;
use crate::vpnd::node::Node;

use super::mixnet_config::{MixnetTrafficConfig, MixnetTrafficDefaults};

use crate::vpnd::gateway::GatewaySelectionAlgorithmConfig;
use crate::vpnd::tunnel::{FrontingMode, GeoExclusionSettings, SplitTunnelSettings};

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct VpndConfig {
    pub entry_node: Node,
    pub exit_node: Node,
    pub custom_dns: Option<Vec<IpAddr>>,
    pub enable_custom_dns: bool,
    pub allow_lan: bool,
    pub enable_ad_blocking: bool,
    pub disable_ipv6: bool,
    pub vpn_mode: VpnMode,
    pub bridges: bool,
    pub netstack: bool,
    pub fronting_mode: FrontingMode,
    pub min_gateway_vpn_performance: Option<u8>,
    pub residential_exit: bool,
    pub mixnet_traffic: MixnetTrafficConfig,
    pub mixnet_traffic_defaults: MixnetTrafficDefaults,
    pub split_tunnel: SplitTunnelSettings,
    pub geo_exclusion: GeoExclusionSettings,
    pub gateway_selection_algorithm_config: GatewaySelectionAlgorithmConfig,
    pub gateway_independence_notifications: bool,
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
            enable_ad_blocking: config.enable_ad_blocking,
            disable_ipv6: config.disable_ipv6,
            vpn_mode,
            bridges: config.enable_bridges,
            netstack: config.netstack,
            fronting_mode: config.fronting_mode.into(),
            min_gateway_vpn_performance: config.min_gateway_vpn_performance,
            residential_exit: config.residential_exit,
            mixnet_traffic: config.mixnet_traffic.into(),
            mixnet_traffic_defaults: MixnetTrafficDefaults::get(),
            split_tunnel: config.split_tunnel.into(),
            geo_exclusion: config.geo_exclusion.into(),
            gateway_selection_algorithm_config: config.gateway_selection_algorithm_config.into(),
            gateway_independence_notifications: config.gateway_independence.enable_notifications,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_with_two_hop_maps_to_wireguard() {
        let cfg = VpndConfig::from_lib(lib::VpnServiceConfig::default())
            .expect("default config should convert");

        // Default `enable_two_hop` is true → Wireguard mode.
        assert_eq!(cfg.vpn_mode, VpnMode::Wg);
        assert!(matches!(cfg.entry_node, Node::Random));
        assert!(matches!(cfg.exit_node, Node::Random));
        assert_eq!(cfg.custom_dns, Some(vec![]));
        assert!(!cfg.enable_custom_dns);
        assert!(!cfg.allow_lan);
        assert!(!cfg.disable_ipv6);
    }

    #[test]
    fn two_hop_disabled_maps_to_mixnet() {
        let lib_cfg = lib::VpnServiceConfig {
            enable_two_hop: false,
            ..Default::default()
        };
        let cfg = VpndConfig::from_lib(lib_cfg).unwrap();
        assert_eq!(cfg.vpn_mode, VpnMode::Mixnet);
    }

    #[test]
    fn passes_through_flags_and_nodes() {
        let ip: IpAddr = "1.1.1.1".parse().unwrap();
        let lib_cfg = lib::VpnServiceConfig {
            entry_point: lib::EntryPoint::Country {
                two_letter_iso_country_code: "DE".to_string(),
            },
            exit_point: lib::ExitPoint::Country {
                two_letter_iso_country_code: "FR".to_string(),
            },
            allow_lan: true,
            disable_ipv6: true,
            enable_ad_blocking: true,
            enable_bridges: true,
            netstack: true,
            enable_custom_dns: true,
            custom_dns: vec![ip],
            min_gateway_vpn_performance: Some(90),
            residential_exit: true,
            ..Default::default()
        };
        let cfg = VpndConfig::from_lib(lib_cfg).unwrap();

        assert!(matches!(cfg.entry_node, Node::Country { ref code } if code == "DE"));
        assert!(matches!(cfg.exit_node, Node::Country { ref code } if code == "FR"));
        assert!(cfg.allow_lan);
        assert!(cfg.disable_ipv6);
        assert!(cfg.enable_ad_blocking);
        assert!(cfg.bridges);
        assert!(cfg.netstack);
        assert!(cfg.enable_custom_dns);
        assert_eq!(cfg.custom_dns, Some(vec![ip]));
        assert_eq!(cfg.min_gateway_vpn_performance, Some(90));
        assert!(cfg.residential_exit);
    }

    // NOTE: the `Err` path (unsupported `ExitPoint::Address`) is not covered — constructing
    // an `ExitPoint::Address { address: Box<Recipient> }` requires valid base58 crypto keys,
    // which is disproportionate for a single passthrough `return Err` on a TODO variant.
}
