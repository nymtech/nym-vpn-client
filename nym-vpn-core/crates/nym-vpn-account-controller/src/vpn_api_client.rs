// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ops::Deref;

use nym_vpn_api_client::{response::NymVpnAccountStatusResponse, types::VpnApiAccount};
use nym_vpn_lib_types::{StoreAccountError, VpnApiErrorResponse};

#[derive(Clone, Debug)]
pub(crate) struct AccountControllerVpnApiClient {
    inner: nym_vpn_api_client::VpnApiClient,
}

impl AccountControllerVpnApiClient {
    pub(crate) fn new(vpn_api_client: nym_vpn_api_client::VpnApiClient) -> Self {
        Self {
            inner: vpn_api_client,
        }
    }

    pub(crate) fn inner(&self) -> &nym_vpn_api_client::VpnApiClient {
        &self.inner
    }

    pub(crate) fn swap_inner_client(&mut self, new_client: nym_vpn_api_client::VpnApiClient) {
        self.inner = new_client;
    }

    pub(crate) async fn check_account_exists_on_api(
        &self,
        account: &VpnApiAccount,
    ) -> Result<(), StoreAccountError> {
        let response = self.inner.get_account(account).await.map_err(|e| {
            VpnApiErrorResponse::try_from(e)
                .map(StoreAccountError::GetAccountEndpointFailure)
                .unwrap_or_else(|e| StoreAccountError::UnexpectedResponse(e.to_string()))
        });

        // TODO: handle these cases
        match response {
            Ok(account) => match account.status {
                NymVpnAccountStatusResponse::Active => Ok(()),
                NymVpnAccountStatusResponse::Inactive => Ok(()),
                NymVpnAccountStatusResponse::DeleteMe => Ok(()),
            },
            Err(err) => Err(err),
        }
    }
}

impl Deref for AccountControllerVpnApiClient {
    type Target = nym_vpn_api_client::VpnApiClient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
