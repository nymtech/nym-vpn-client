// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ops::Deref;

use nym_vpn_api_client::{response::NymVpnAccountStatusResponse, types::VpnApiAccount};
use nym_vpn_lib_types::{StoreAccountError, VpnApiErrorResponse};

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

    pub(crate) async fn check_account_exists_on_api(
        &self,
        account: &VpnApiAccount,
    ) -> Result<(), StoreAccountError> {
        let response = self.inner.get_account(account).await.map_err(|e| {
            VpnApiErrorResponse::try_from(e)
                .map(StoreAccountError::GetAccountEndpointFailure)
                .unwrap_or_else(StoreAccountError::unexpected_response)
        });

        // TODO: handle these cases
        // The logic below replicates the previous behaviour, but we should extend it to also
        // handle where the account exists, but is not active or soft-deleted.
        match response {
            Ok(account) => match account.status {
                NymVpnAccountStatusResponse::Active => Ok(()),
                NymVpnAccountStatusResponse::Inactive => {
                    tracing::warn!("Account is inactive - proceeding anyway");
                    Ok(())
                }
                NymVpnAccountStatusResponse::DeleteMe => {
                    tracing::warn!("Account is marked for deletion - proceeding anyway");
                    Ok(())
                }
            },
            Err(err) => Err(err),
        }
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
