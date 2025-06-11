// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::env;

use nym_config::defaults::{NymNetworkDetails, var_names};
use url::Url;

use crate::{
    AccountManagement, ParsedAccountLinks, Result, SystemMessages,
    account_management::AccountLinksConversionError, discovery::Discovery,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NymVpnNetwork {
    pub nym_vpn_api_url: Url,
    pub account_management: Option<AccountManagement>,
    pub system_messages: SystemMessages,
}

impl NymVpnNetwork {
    pub fn new(network_details: NymNetworkDetails) -> Self {
        // These expects are safe because we are using the hardcoded mainnet defaults
        #[allow(clippy::expect_used)]
        Self {
            nym_vpn_api_url: network_details
                .nym_vpn_api_url
                .expect("mainnet default for nym_vpn_api_url is missing")
                .parse()
                .expect("mainnet default for nym_vpn_api_url is invalid"),
            account_management: None,
            system_messages: SystemMessages::default(),
        }
    }
    pub(super) fn export_to_env(&self) {
        // todo: prefer dependency injection to env variable.
        unsafe { env::set_var(var_names::NYM_VPN_API, self.nym_vpn_api_url.to_string()) };
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
        Self {
            nym_vpn_api_url: discovery.nym_vpn_api_url,
            account_management: discovery.account_management,
            system_messages: discovery.system_messages,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NymVpnNetworkFromDetailsError {
    #[error("Nym vpn api url is missing in the network details")]
    NymVpnApiUrlMissing,

    #[error("Failed to parse Nym VPN API URL")]
    ParseNymVpnApiUrlError(#[source] url::ParseError),
}

impl TryFrom<&NymNetworkDetails> for NymVpnNetwork {
    type Error = NymVpnNetworkFromDetailsError;

    fn try_from(network_details: &NymNetworkDetails) -> Result<Self, Self::Error> {
        let nym_vpn_api_url = network_details
            .nym_vpn_api_url
            .clone()
            .ok_or(NymVpnNetworkFromDetailsError::NymVpnApiUrlMissing)?
            .parse()
            .map_err(NymVpnNetworkFromDetailsError::ParseNymVpnApiUrlError)?;

        Ok(Self {
            nym_vpn_api_url,
            account_management: None,
            system_messages: SystemMessages::default(),
        })
    }
}
