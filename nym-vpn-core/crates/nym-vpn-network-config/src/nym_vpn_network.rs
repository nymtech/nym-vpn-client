// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::env;

use nym_network_defaults::{ApiUrl, NymNetworkDetails, var_names};

use crate::{
    AccountManagement, ParsedAccountLinks, Result, SystemMessages,
    account_management::AccountLinksConversionError, discovery::Discovery,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NymVpnNetwork {
    pub nym_vpn_api_urls: Vec<ApiUrl>,
    pub account_management: Option<AccountManagement>,
    pub system_messages: SystemMessages,
}

impl NymVpnNetwork {
    pub fn new(network_details: NymNetworkDetails) -> Self {
        let nym_vpn_api_urls = network_details.nym_vpn_api_urls();

        Self {
            nym_vpn_api_urls,
            account_management: None,
            system_messages: SystemMessages::default(),
        }
    }

    pub(super) fn export_to_env(&self) {
        if let Some(api_url) = self.nym_vpn_api_urls.first() {
            unsafe { env::set_var(var_names::NYM_VPN_API, &api_url.url) }
        } else {
            tracing::warn!("Nym VPN API URL missing");
        }
    }

    pub fn try_into_parsed_links(
        self,
        locale: &str,
        account_id: Option<&str>,
    ) -> Result<ParsedAccountLinks> {
        let account_management = self
            .account_management
            .ok_or(NymVpnNetworkAccountLinksConversionError::Unavailable)?;

        Ok(account_management
            .try_into_parsed_links(locale, account_id)
            .map_err(NymVpnNetworkAccountLinksConversionError::Conversion)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NymVpnNetworkAccountLinksConversionError {
    #[error("account management is not available for this network")]
    Unavailable,

    #[error(transparent)]
    Conversion(AccountLinksConversionError),
}

impl From<Discovery> for NymVpnNetwork {
    fn from(discovery: Discovery) -> Self {
        let nym_vpn_api_urls = discovery.nym_vpn_api_urls().clone();

        Self {
            nym_vpn_api_urls,
            account_management: discovery.account_management,
            system_messages: discovery.system_messages,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NymVpnNetworkFromDetailsError {
    #[error("Nym vpn api url is missing in the network details")]
    NymVpnApiUrlMissing,

    #[error("Nym vpn api urls are missing in the network details")]
    NymVpnApiUrlsMissing,

    #[error("Failed to parse Nym VPN API URL")]
    ParseNymVpnApiUrlError(#[source] url::ParseError),
}

impl TryFrom<&NymNetworkDetails> for NymVpnNetwork {
    type Error = NymVpnNetworkFromDetailsError;

    fn try_from(network_details: &NymNetworkDetails) -> Result<Self, Self::Error> {
        let nym_vpn_api_urls = network_details.nym_vpn_api_urls();
        if nym_vpn_api_urls.is_empty() {
            return Err(NymVpnNetworkFromDetailsError::NymVpnApiUrlsMissing);
        }

        Ok(Self {
            nym_vpn_api_urls,
            account_management: None,
            system_messages: SystemMessages::default(),
        })
    }
}
