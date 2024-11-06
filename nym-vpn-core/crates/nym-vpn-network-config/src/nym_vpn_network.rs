// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::env;

use nym_config::defaults::{var_names, NymNetworkDetails};
use url::Url;

use crate::{AccountManagement, SystemMessages};

use super::discovery::Discovery;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NymVpnNetwork {
    pub nym_vpn_api_url: Url,
    pub account_management: Option<AccountManagement>,
    pub system_messages: SystemMessages,
}

impl NymVpnNetwork {
    pub(super) fn export_to_env(&self) {
        env::set_var(var_names::NYM_VPN_API, self.nym_vpn_api_url.to_string());
    }
}

impl From<Discovery> for NymVpnNetwork {
    fn from(discovery: Discovery) -> Self {
        Self {
            nym_vpn_api_url: discovery.nym_vpn_api_url,
            account_management: discovery.account_management,
            system_messages: discovery.system_messages,
        }
    }
}

impl TryFrom<&NymNetworkDetails> for NymVpnNetwork {
    type Error = anyhow::Error;

    fn try_from(network_details: &NymNetworkDetails) -> Result<Self, Self::Error> {
        let nym_vpn_api_url = network_details
            .nym_vpn_api_url
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Failed to find NYM_VPN_API_URL in the environment"))?
            .parse()?;

        Ok(Self {
            nym_vpn_api_url,
            account_management: None,
            system_messages: SystemMessages::default(),
        })
    }
}
