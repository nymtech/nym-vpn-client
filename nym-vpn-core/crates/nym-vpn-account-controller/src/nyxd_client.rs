// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_validator_client::{
    DirectSigningHttpRpcNyxdClient,
    nyxd::{Coin, Config, CosmWasmClient, cosmwasm_client::types::Account},
};
use nym_vpn_lib_types::{AccountCommandError, Mnemonic};
use nym_vpn_network_config::Network;
use std::{str::FromStr, sync::Arc};

pub struct NyxdClient {
    network: Network,

    // TODO: does it need locking or are we guaranteed sequential access?
    client: Option<Arc<DirectSigningHttpRpcNyxdClient>>,
}

impl NyxdClient {
    pub fn new(network: &Network) -> Self {
        NyxdClient {
            network: network.clone(),
            client: None,
        }
    }

    pub(crate) fn ensure_connected(&mut self, mnemonic: &str) -> Result<(), AccountCommandError> {
        if self.client.is_some() {
            return Ok(());
        }
        let network_details: nym_sdk::NymNetworkDetails =
            self.network.nym_network_details().clone().into();
        let client_config =
            Config::try_from_nym_network_details(&network_details).map_err(|err| {
                AccountCommandError::NyxdConnectionFailure(format!(
                    "invalid network information: {err}"
                ))
            })?;

        let client = DirectSigningHttpRpcNyxdClient::connect_with_mnemonic(
            client_config,
            self.network.nyxd_url.as_str(),
            Mnemonic::from_str(mnemonic)
                .map_err(|err| AccountCommandError::InvalidMnemonic(err.to_string()))?,
        )
        .map_err(|err| AccountCommandError::NyxdConnectionFailure(err.to_string()))?;

        self.client = Some(Arc::new(client));
        Ok(())
    }

    pub(crate) fn disconnect(&mut self) {
        self.client = None;
    }

    pub(crate) async fn get_account_details(
        &mut self,
        mnemonic: &str,
    ) -> Result<Option<Account>, AccountCommandError> {
        self.ensure_connected(mnemonic)?;
        // SAFETY: we just connected
        #[allow(clippy::unwrap_used)]
        let client = self.client.as_mut().unwrap();
        let address = client.address();
        client
            .get_account(&address)
            .await
            .map_err(|err| AccountCommandError::NyxdQueryFailure(err.to_string()))
    }

    pub(crate) async fn account_balance(
        &mut self,
        mnemonic: &str,
    ) -> Result<Vec<Coin>, AccountCommandError> {
        self.ensure_connected(mnemonic)?;
        // SAFETY: we just connected
        #[allow(clippy::unwrap_used)]
        let client = self.client.as_mut().unwrap();
        let address = client.address();
        client
            .get_all_balances(&address)
            .await
            .map_err(|err| AccountCommandError::NyxdQueryFailure(err.to_string()))
    }

    pub(crate) fn inner_client(
        &mut self,
        mnemonic: &str,
    ) -> Result<Arc<DirectSigningHttpRpcNyxdClient>, AccountCommandError> {
        self.ensure_connected(mnemonic)?;
        // SAFETY: we just connected
        #[allow(clippy::unwrap_used)]
        let client = self.client.clone().unwrap();
        Ok(client)
    }
}
