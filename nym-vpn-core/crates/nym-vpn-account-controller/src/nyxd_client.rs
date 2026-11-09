// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Error, Result};
use nym_validator_client::{
    DirectSigningHttpRpcNyxdClient,
    nyxd::{Coin, Config, TendermintRpcClientExt, cosmwasm_client::types::Account},
};
use nym_vpn_lib_types::Mnemonic;
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

    pub(crate) fn ensure_connected(&mut self, mnemonic: &str) -> Result<()> {
        if self.client.is_some() {
            return Ok(());
        }
        let network_details = self.network.nym_network_details().clone();
        let client_config =
            Config::try_from_nym_network_details(&network_details).map_err(|err| {
                Error::NyxdConnectionFailure(format!("invalid network information: {err}").into())
            })?;

        let client = DirectSigningHttpRpcNyxdClient::connect_with_mnemonic(
            client_config,
            self.network.nyxd_url.as_str(),
            Mnemonic::from_str(mnemonic).map_err(|err| Error::InvalidMnemonic(err.into()))?,
        )
        .map_err(|err| Error::NyxdConnectionFailure(err.into()))?;

        self.client = Some(Arc::new(client));
        Ok(())
    }

    pub(crate) fn disconnect(&mut self) {
        self.client = None;
    }

    pub(crate) async fn get_account_details(&mut self, mnemonic: &str) -> Result<Option<Account>> {
        self.ensure_connected(mnemonic)?;
        // SAFETY: we just connected
        #[allow(clippy::unwrap_used)]
        let client = self.client.as_mut().unwrap();
        let address = client.address();
        client
            .get_account(&address)
            .await
            .map_err(|err| Error::NyxdQueryFailure(err.into()))
    }

    pub(crate) async fn account_balance(&mut self, mnemonic: &str) -> Result<Vec<Coin>> {
        self.ensure_connected(mnemonic)?;
        // SAFETY: we just connected
        #[allow(clippy::unwrap_used)]
        let client = self.client.as_mut().unwrap();
        let address = client.address();
        client
            .get_all_balances(&address)
            .await
            .map_err(|err| Error::NyxdQueryFailure(err.into()))
    }

    pub(crate) fn inner_client(
        &mut self,
        mnemonic: &str,
    ) -> Result<Arc<DirectSigningHttpRpcNyxdClient>> {
        self.ensure_connected(mnemonic)?;
        // SAFETY: we just connected
        #[allow(clippy::unwrap_used)]
        let client = self.client.clone().unwrap();
        Ok(client)
    }
}
