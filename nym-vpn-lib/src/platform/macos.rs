// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::gateway_client::{EntryPoint, ExitPoint};
use crate::NymVpn;
use log::warn;
use oslog::OsLogger;
use std::net::{Ipv4Addr, Ipv6Addr};
use talpid_types::net::wireguard::{PeerConfig, TunnelConfig};
use url::Url;

fn init_logs() {
    OsLogger::new("net.nymtech.vpn.agent")
        .level_filter(LevelFilter::Debug)
        .category_level_filter("hyper", log::LevelFilter::Warn)
        .category_level_filter("tokio_reactor", log::LevelFilter::Warn)
        .category_level_filter("reqwest", log::LevelFilter::Warn)
        .category_level_filter("mio", log::LevelFilter::Warn)
        .category_level_filter("want", log::LevelFilter::Warn)
        .category_level_filter("tungstenite", log::LevelFilter::Warn)
        .category_level_filter("tokio_tungstenite", log::LevelFilter::Warn)
        .category_level_filter("handlebars", log::LevelFilter::Warn)
        .category_level_filter("sled", log::LevelFilter::Warn)
        .init()
        .expect("Could not init logs");
    debug!("Logger initialized");
}

#[derive(Clone)]
pub struct WgConfig {
    pub tunnel: TunnelConfig,
    pub peers: Vec<PeerConfig>,
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Option<Ipv6Addr>,
    pub mtu: u16,
}

pub struct VPNConfig {
    pub api_url: Url,
    pub explorer_url: Url,
    pub entry_gateway: EntryPoint,
    pub exit_router: ExitPoint,
}

#[allow(non_snake_case)]
pub async fn initVPN(config: VPNConfig) {
    init_logs();

    if get_vpn_state().await != ClientState::Uninitialised {
        warn!("VPN was already inited. Try starting it");
        return;
    }

    let mut vpn = NymVpn::new(config.entry_gateway, config.exit_router);
    vpn.gateway_config.api_url = config.api_url;
    vpn.gateway_config.explorer_url = Some(config.explorer_url);

    set_inited_vpn(vpn).await
}
