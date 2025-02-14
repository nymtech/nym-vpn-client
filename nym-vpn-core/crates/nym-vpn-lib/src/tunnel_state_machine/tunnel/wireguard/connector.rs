// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_network_config::Network;
use tokio::task::JoinHandle;

use nym_authenticator_client::AuthClient;
use nym_credentials_interface::TicketType;
use nym_gateway_directory::{AuthAddresses, Gateway, GatewayClient};
use nym_mixnet_client::SharedMixnetClient;
use nym_sdk::mixnet::{ConnectionStatsEvent, EphemeralCredentialStorage, StoragePaths};
use nym_task::TaskManager;
use nym_wg_gateway_client::{GatewayData, WgGatewayClient};
use tokio_util::sync::CancellationToken;

use super::connected_tunnel::ConnectedTunnel;
use crate::{
    bandwidth_controller::BandwidthController,
    tunnel_state_machine::tunnel::{
        self, gateway_selector::SelectedGateways, AnyConnector, ConnectorError, Error, Result,
    },
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
        network: &Network,
        enable_credentials_mode: bool,
        selected_gateways: SelectedGateways,
        data_path: Option<PathBuf>,
        cancel_token: CancellationToken,
    ) -> Result<ConnectedTunnel, ConnectorError> {
        let result = Self::connect_inner(
            &self.task_manager,
            network,
            self.mixnet_client.clone(),
            &self.gateway_directory_client,
            enable_credentials_mode,
            selected_gateways,
            data_path,
            cancel_token,
        )
        .await;

        match result {
            Ok(connect_result) => Ok(ConnectedTunnel::new(
                self.task_manager,
                connect_result.entry_gateway_client,
                connect_result.exit_gateway_client,
                connect_result.connection_data,
                connect_result.bandwidth_controller_handle,
            )),
            Err(e) => Err(ConnectorError::new(
                e,
                AnyConnector::Wireguard(Self::new(
                    self.task_manager,
                    self.mixnet_client,
                    self.gateway_directory_client,
                )),
            )),
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn connect_inner(
        task_manager: &TaskManager,
        network: &Network,
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: &GatewayClient,
        enable_credentials_mode: bool,
        selected_gateways: SelectedGateways,
        data_path: Option<PathBuf>,
        cancel_token: CancellationToken,
    ) -> Result<ConnectResult> {
        let auth_addresses =
            Self::setup_auth_addresses(&selected_gateways.entry, &selected_gateways.exit)?;
        let (Some(entry_auth_recipient), Some(exit_auth_recipient)) =
            (auth_addresses.entry().0, auth_addresses.exit().0)
        else {
            return Err(Error::AuthenticationNotPossible(auth_addresses.to_string()));
        };
        let entry_version = selected_gateways.entry.version.clone().into();
        let exit_version = selected_gateways.exit.version.clone().into();
        let auth_client = AuthClient::new(mixnet_client).await;

        let mut wg_entry_gateway_client = if enable_credentials_mode {
            WgGatewayClient::new_free_entry(
                &data_path,
                auth_client.clone(),
                entry_auth_recipient,
                entry_version,
            )
        } else {
            WgGatewayClient::new_entry(
                &data_path,
                auth_client.clone(),
                entry_auth_recipient,
                entry_version,
            )
        };
        let mut wg_exit_gateway_client = if enable_credentials_mode {
            WgGatewayClient::new_free_exit(
                &data_path,
                auth_client.clone(),
                exit_auth_recipient,
                exit_version,
            )
        } else {
            WgGatewayClient::new_exit(
                &data_path,
                auth_client.clone(),
                exit_auth_recipient,
                exit_version,
            )
        };

        let shutdown = task_manager.subscribe_named("bandwidth_controller");
        let (connection_data, bandwidth_controller_handle) = if let Some(data_path) =
            data_path.as_ref()
        {
            let paths = StoragePaths::new_from_dir(data_path).map_err(Error::SetupStoragePaths)?;
            let storage = paths
                .persistent_credential_storage()
                .await
                .map_err(Error::SetupStoragePaths)?;
            let bw = BandwidthController::new(
                storage,
                network,
                wg_entry_gateway_client.light_client(),
                wg_exit_gateway_client.light_client(),
                shutdown,
            )?;
            let entry_fut = bw.get_initial_bandwidth(
                enable_credentials_mode,
                TicketType::V1WireguardEntry,
                gateway_directory_client,
                &mut wg_entry_gateway_client,
            );
            let exit_fut = bw.get_initial_bandwidth(
                enable_credentials_mode,
                TicketType::V1WireguardExit,
                gateway_directory_client,
                &mut wg_exit_gateway_client,
            );
            let entry = cancel_token
                .run_until_cancelled(entry_fut)
                .await
                .ok_or(tunnel::Error::Cancelled)??;
            let exit = cancel_token
                .run_until_cancelled(exit_fut)
                .await
                .ok_or(tunnel::Error::Cancelled)??;

            let bandwidth_controller_handle = tokio::spawn(bw.run());

            (ConnectionData { entry, exit }, bandwidth_controller_handle)
        } else {
            let storage = EphemeralCredentialStorage::default();
            let bw = BandwidthController::new(
                storage,
                network,
                wg_entry_gateway_client.light_client(),
                wg_exit_gateway_client.light_client(),
                shutdown,
            )?;
            let entry = bw
                .get_initial_bandwidth(
                    enable_credentials_mode,
                    TicketType::V1WireguardEntry,
                    gateway_directory_client,
                    &mut wg_entry_gateway_client,
                )
                .await?;
            let exit = bw
                .get_initial_bandwidth(
                    enable_credentials_mode,
                    TicketType::V1WireguardExit,
                    gateway_directory_client,
                    &mut wg_exit_gateway_client,
                )
                .await?;

            let bandwidth_controller_handle = tokio::spawn(bw.run());

            (ConnectionData { entry, exit }, bandwidth_controller_handle)
        };

        if let Some(exit_country_code) = selected_gateways.exit.two_letter_iso_country_code() {
            auth_client.send_stats_event(
                ConnectionStatsEvent::WgCountry(exit_country_code.to_string()).into(),
            );
        }

        Ok(ConnectResult {
            entry_gateway_client: wg_entry_gateway_client,
            exit_gateway_client: wg_exit_gateway_client,
            connection_data,
            bandwidth_controller_handle,
        })
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

    /// Gracefully shutdown task manager and mixnet client, and consume the struct.
    pub async fn dispose(self) {
        tracing::debug!("Shutting down mixnet client");
        tunnel::shutdown_mixnet_client(self.task_manager, self.mixnet_client).await;
    }
}

struct ConnectResult {
    entry_gateway_client: WgGatewayClient,
    exit_gateway_client: WgGatewayClient,
    connection_data: ConnectionData,
    bandwidth_controller_handle: JoinHandle<()>,
}
