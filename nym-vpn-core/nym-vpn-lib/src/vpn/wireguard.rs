// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};

use nym_gateway_directory::{EntryPoint, ExitPoint, NodeIdentity};
use talpid_tunnel::tun_provider::TunProvider;

#[cfg(target_os = "ios")]
use crate::platform::swift::OSTunProvider;

use super::{
    base::{GenericNymVpnConfig, ShadowHandle, Vpn},
    MixnetClientConfig, NymVpn,
};

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
        #[cfg(target_os = "android")] android_context: talpid_types::android::AndroidContext,
        #[cfg(target_os = "ios")] ios_tun_provider: Arc<dyn OSTunProvider>,
    ) -> Self {
        let tun_provider = Arc::new(Mutex::new(TunProvider::new(
            #[cfg(target_os = "android")]
            android_context,
            #[cfg(target_os = "android")]
            false,
            #[cfg(target_os = "android")]
            None,
            #[cfg(target_os = "android")]
            vec![],
        )));

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
            #[cfg(target_os = "ios")]
            ios_tun_provider,
            shadow_handle: ShadowHandle { _inner: None },
        }
    }
}
