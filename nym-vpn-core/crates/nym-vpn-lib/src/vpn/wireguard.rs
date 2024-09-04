// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};

use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity};
use talpid_tunnel::tun_provider::TunProvider;

use super::{
    base::{GenericNymVpnConfig, ShadowHandle, Vpn},
    MixnetClientConfig, NymVpn,
};
#[cfg(target_os = "ios")]
use crate::mobile::ios::tun_provider::OSTunProvider;
#[cfg(target_os = "android")]
use crate::platform::android::AndroidTunProvider;

#[derive(Clone, Debug)]
pub struct WireguardConnectionInfo {
    pub gateway_id: NodeIdentity,
    pub public_key: String,
    pub private_ipv4: Ipv4Addr,
}

pub struct WireguardVpn {}

impl Vpn for WireguardVpn {}

impl NymVpn<WireguardVpn> {
    pub fn new_wireguard_vpn(
        entry_point: EntryPoint,
        exit_point: ExitPoint,
        #[cfg(target_os = "android")] android_tun_provider: Arc<dyn AndroidTunProvider>,
        #[cfg(target_os = "ios")] ios_tun_provider: Arc<dyn OSTunProvider>,
    ) -> Self {
        let tun_provider = Arc::new(Mutex::new(TunProvider::new()));

        Self {
            generic_config: GenericNymVpnConfig {
                mixnet_client_config: MixnetClientConfig {
                    enable_poisson_rate: false,
                    disable_background_cover_traffic: false,
                    enable_credentials_mode: false,
                    min_mixnode_performance: None,
                    min_gateway_performance: None,
                },
                data_path: None,
                gateway_config: nym_gateway_directory::Config::default(),
                entry_point,
                exit_point,
                nym_ips: None,
                nym_mtu: None,
                dns: None,
                disable_routing: false,
                user_agent: None,
            },
            vpn_config: WireguardVpn {},
            tun_provider,
            #[cfg(target_os = "android")]
            android_tun_provider,
            #[cfg(target_os = "ios")]
            ios_tun_provider,
            shadow_handle: ShadowHandle { _inner: None },
        }
    }
}
