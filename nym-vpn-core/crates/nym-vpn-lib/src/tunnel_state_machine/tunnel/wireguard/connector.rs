// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_authenticator_client::AuthClient;
use nym_credentials_interface::TicketType;
use nym_gateway_directory::{AuthAddresses, Gateway, GatewayClient};
use nym_sdk::mixnet::{EphemeralCredentialStorage, StoragePaths};
use nym_task::TaskManager;
use nym_wg_gateway_client::{GatewayData, WgGatewayClient};

use super::connected_tunnel::ConnectedTunnel;
use crate::{
    bandwidth_controller::{get_nyxd_client, BandwidthController},
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

        let client = get_nyxd_client().map_err(|e| Error::NyxdSetup {
            reason: e.to_string(),
        })?;
        let shutdown = self.task_manager.subscribe_named("bandwidth controller");
        let (connection_data, bandwidth_controller_handle) =
            if let Some(data_path) = data_path.as_ref() {
                let paths = StoragePaths::new_from_dir(data_path)?;
                let storage = paths.persistent_credential_storage().await?;
                let inner = nym_bandwidth_controller::BandwidthController::new(storage, client);
                let bw = BandwidthController::new(
                    inner,
                    wg_entry_gateway_client.light_client(),
                    wg_exit_gateway_client.light_client(),
                    shutdown,
                );
                let entry = bw
                    .get_initial_bandwidth(
                        TicketType::V1WireguardEntry,
                        &self.gateway_directory_client,
                        &mut wg_entry_gateway_client,
                    )
                    .await?;
                let exit = bw
                    .get_initial_bandwidth(
                        TicketType::V1WireguardExit,
                        &self.gateway_directory_client,
                        &mut wg_exit_gateway_client,
                    )
                    .await?;

                let bandwidth_controller_handle = tokio::spawn(bw.run());

                (ConnectionData { entry, exit }, bandwidth_controller_handle)
            } else {
                let storage = EphemeralCredentialStorage::default();
                let inner = nym_bandwidth_controller::BandwidthController::new(storage, client);
                let bw = BandwidthController::new(
                    inner,
                    wg_entry_gateway_client.light_client(),
                    wg_exit_gateway_client.light_client(),
                    shutdown,
                );
                let entry = bw
                    .get_initial_bandwidth(
                        TicketType::V1WireguardEntry,
                        &self.gateway_directory_client,
                        &mut wg_entry_gateway_client,
                    )
                    .await?;
                let exit = bw
                    .get_initial_bandwidth(
                        TicketType::V1WireguardExit,
                        &self.gateway_directory_client,
                        &mut wg_exit_gateway_client,
                    )
                    .await?;

                let bandwidth_controller_handle = tokio::spawn(bw.run());

                (ConnectionData { entry, exit }, bandwidth_controller_handle)
            };

        Ok(ConnectedTunnel::new(
            self.task_manager,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            connection_data,
            bandwidth_controller_handle,
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
}
