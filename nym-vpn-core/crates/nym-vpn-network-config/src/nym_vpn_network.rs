// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::env;

use nym_network_defaults::{ApiUrl, NymNetworkDetails, var_names};
use url::Url;

use crate::{
    AccountManagement, ParsedAccountLinks, Result, SystemMessages,
    account_management::AccountLinksConversionError, discovery::Discovery,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NymVpnNetwork {
    pub nym_vpn_api_url: Url,
    pub nym_vpn_api_urls: Vec<ApiUrl>,
    pub account_management: Option<AccountManagement>,
    pub system_messages: SystemMessages,
}

impl NymVpnNetwork {
    pub fn new(network_details: NymNetworkDetails) -> Self {
        // TODO: refactor out this junk
        #[allow(clippy::expect_used)]
        Self {
            nym_vpn_api_url: network_details
                .nym_vpn_api_url
                .expect("nym_vpn_api_url is missing")
                .parse()
                .expect("nym_vpn_api_url is invalid"),
            nym_vpn_api_urls: network_details
                .nym_vpn_api_urls
                .unwrap_or_default()
                .into_iter()
                .collect(),
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

    // Picks the first URL with fronting configured
    pub fn fronted_vpn_api_url(&self) -> Option<ApiUrl> {
        if let Some(api_url) = self
            .nym_vpn_api_urls
            .iter()
            .find(|u| u.front_hosts.as_ref().is_some_and(|f| !f.is_empty()))
        {
            return Some(api_url.clone());
        }

        if let Some(api_url) = self.nym_vpn_api_urls.first() {
            return Some(api_url.clone());
        }

        None
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
        let nym_vpn_api_urls = discovery
            .nym_vpn_api_urls
            .iter()
            .map(|u| nym_network_defaults::ApiUrl {
                url: u.url.clone(),
                front_hosts: u.fronts.clone(),
            })
            .collect();

        Self {
            nym_vpn_api_url: discovery.nym_vpn_api_url,
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
        let nym_vpn_api_url = network_details
            .nym_vpn_api_url
            .clone()
            .ok_or(NymVpnNetworkFromDetailsError::NymVpnApiUrlMissing)?
            .parse()
            .map_err(NymVpnNetworkFromDetailsError::ParseNymVpnApiUrlError)?;
        let nym_vpn_api_urls = network_details
            .nym_vpn_api_urls
            .clone()
            .ok_or(NymVpnNetworkFromDetailsError::NymVpnApiUrlsMissing)?
            .iter()
            .map(|api_url| Ok(api_url.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            nym_vpn_api_url,
            nym_vpn_api_urls,
            account_management: None,
            system_messages: SystemMessages::default(),
        })
    }
}
