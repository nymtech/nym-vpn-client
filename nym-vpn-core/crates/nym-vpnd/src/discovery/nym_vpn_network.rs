// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::env;

use nym_vpn_lib::nym_config::defaults::var_names;
use url::Url;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct NymVpnNetworkDetails {
    pub(super) nym_vpn_api_url: String,
}

pub(super) fn setup_nym_vpn_network_details(
    nym_vpn_api_url: Url,
) -> anyhow::Result<NymVpnNetworkDetails> {
    let vpn_network_details = NymVpnNetworkDetails {
        nym_vpn_api_url: nym_vpn_api_url.to_string(),
    };
    export_nym_vpn_network_details_to_env(vpn_network_details.clone());
    Ok(vpn_network_details)
}

fn export_nym_vpn_network_details_to_env(vpn_network_details: NymVpnNetworkDetails) {
    env::set_var(var_names::NYM_VPN_API, vpn_network_details.nym_vpn_api_url);
}
