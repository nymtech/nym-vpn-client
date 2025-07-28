// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ops::Deref;

use crate::{AccountControllerConfig, Error};

#[derive(Clone, Debug)]
pub(crate) struct AccountControllerVpnApiClient {
    inner: nym_vpn_api_client::VpnApiClient,
}

impl AccountControllerVpnApiClient {
    pub(crate) fn new(config: &AccountControllerConfig) -> Result<Self, Error> {
        nym_vpn_api_client::VpnApiClient::new(
            config.network_env.vpn_api_url(),
            config.user_agent.clone(),
        )
        .map_err(Error::SetupVpnApiClient)
        .map(AccountControllerVpnApiClient::from)
    }

    pub(crate) fn inner(&self) -> &nym_vpn_api_client::VpnApiClient {
        &self.inner
    }

    pub(crate) fn swap_inner_client(&mut self, new_client: nym_vpn_api_client::VpnApiClient) {
        self.inner = new_client;
    }
}

impl From<nym_vpn_api_client::VpnApiClient> for AccountControllerVpnApiClient {
    fn from(vpn_api_client: nym_vpn_api_client::VpnApiClient) -> Self {
        Self {
            inner: vpn_api_client,
        }
    }
}

impl Deref for AccountControllerVpnApiClient {
    type Target = nym_vpn_api_client::VpnApiClient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
