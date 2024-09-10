// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_authenticator_client::AuthClient;
use nym_gateway_directory::{AuthAddresses, Gateway, GatewayClient};
use nym_task::TaskManager;
use nym_wg_gateway_client::{GatewayData, WgGatewayClient};

use super::connected_tunnel::ConnectedTunnel;
use crate::{
    mixnet::SharedMixnetClient,
    tunnel_state_machine::tunnel::{gateway_selector::SelectedGateways, Error, Result},
};

pub struct ConnectionData {
    pub entry: GatewayData,
    pub exit: GatewayData,
}

pub struct Connector {
    task_manager: TaskManager,
    mixnet_client: SharedMixnetClient,
    gateway_directory_client: GatewayClient,
}

impl Connector {
    pub fn new(
        task_manager: TaskManager,
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: GatewayClient,
    ) -> Self {
        Self {
            task_manager,
            mixnet_client,
            gateway_directory_client,
        }
    }

    pub async fn connect(
        self,
        selected_gateways: SelectedGateways,
        data_path: Option<PathBuf>,
    ) -> Result<ConnectedTunnel> {
        let auth_addresses =
            Self::setup_auth_addresses(&selected_gateways.entry, &selected_gateways.exit)?;
        let (Some(entry_auth_recipient), Some(exit_auth_recipient)) =
            (auth_addresses.entry().0, auth_addresses.exit().0)
        else {
            return Err(Error::AuthenticationNotPossible(auth_addresses.to_string()));
        };
        let auth_client = AuthClient::new_from_inner(self.mixnet_client.inner()).await;

        let mut wg_entry_gateway_client =
            WgGatewayClient::new_entry(&data_path, auth_client.clone(), entry_auth_recipient);
        let mut wg_exit_gateway_client =
            WgGatewayClient::new_entry(&data_path, auth_client.clone(), exit_auth_recipient);

        let entry_gateway_data = self.register_wg_key(&mut wg_entry_gateway_client).await?;
        let exit_gateway_data = self.register_wg_key(&mut wg_exit_gateway_client).await?;

        if wg_entry_gateway_client.suspended().await? || wg_exit_gateway_client.suspended().await? {
            return Err(Error::NotEnoughBandwidth);
        }

        let connection_data = ConnectionData {
            entry: entry_gateway_data,
            exit: exit_gateway_data,
        };

        Ok(ConnectedTunnel::new(
            self.task_manager,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            connection_data,
        ))
    }

    fn setup_auth_addresses(entry: &Gateway, exit: &Gateway) -> Result<AuthAddresses> {
        let entry_authenticator_address = entry
            .authenticator_address
            .ok_or(Error::AuthenticatorAddressNotFound)?;
        let exit_authenticator_address = exit
            .authenticator_address
            .ok_or(Error::AuthenticatorAddressNotFound)?;
        Ok(AuthAddresses::new(
            entry_authenticator_address,
            exit_authenticator_address,
        ))
    }

    async fn register_wg_key(
        &self,
        wg_gateway_client: &mut WgGatewayClient,
    ) -> Result<GatewayData> {
        // First we need to register with the gateway to setup keys and IP assignment
        tracing::info!("Registering with wireguard gateway");
        let gateway_id = wg_gateway_client
            .auth_recipient()
            .gateway()
            .to_base58_string();
        let gateway_host = self
            .gateway_directory_client
            .lookup_gateway_ip(&gateway_id)
            .await
            .map_err(|source| Error::FailedToLookupGatewayIp { gateway_id, source })?;
        let wg_gateway_data = wg_gateway_client.register_wireguard(gateway_host).await?;
        tracing::debug!("Received wireguard gateway data: {wg_gateway_data:?}");
        Ok(wg_gateway_data)
    }
}
