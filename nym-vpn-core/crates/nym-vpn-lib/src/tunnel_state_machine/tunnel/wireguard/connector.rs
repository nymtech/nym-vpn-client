// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_network_config::Network;
use tokio::task::JoinHandle;

use nym_authenticator_client::{AuthClientMixnetListener, AuthClientMixnetListenerHandle};
use nym_credentials_interface::TicketType;
use nym_gateway_directory::{AuthAddresses, CachingGatewayClient, Gateway};
use nym_sdk::mixnet::{ConnectionStatsEvent, EphemeralCredentialStorage, StoragePaths};
use nym_task::TaskManager;
use nym_wg_gateway_client::{GatewayData, WgGatewayClient};
use tokio_util::sync::CancellationToken;

use super::connected_tunnel::ConnectedTunnel;
use crate::{
    bandwidth_controller::BandwidthController,
    mixnet::SharedMixnetClient,
    tunnel_state_machine::tunnel::{self, Error, Result, gateway_selector::SelectedGateways},
};

pub struct ConnectionData {
    pub entry: GatewayData,
    pub exit: GatewayData,
}

pub struct Connector {
    mixnet_client: SharedMixnetClient,
    gateway_directory_client: CachingGatewayClient,
}

impl Connector {
    pub fn new(
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: CachingGatewayClient,
    ) -> Self {
        Self {
            mixnet_client,
            gateway_directory_client,
        }
    }

    pub async fn connect(
        self,
        task_manager: &TaskManager,
        network: &Network,
        selected_gateways: SelectedGateways,
        data_path: Option<PathBuf>,
        cancel_token: CancellationToken,
    ) -> Result<ConnectedTunnel> {
        let connect_result = Box::pin(Self::connect_inner(
            task_manager,
            network,
            self.mixnet_client.clone(),
            self.gateway_directory_client.clone(),
            selected_gateways,
            data_path,
            cancel_token,
        ))
        .await?;

        Ok(ConnectedTunnel::new(
            connect_result.entry_gateway_client,
            connect_result.exit_gateway_client,
            connect_result.connection_data,
            connect_result.bandwidth_controller_handle,
            connect_result.auth_client_mixnet_listener_handle,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    async fn connect_inner(
        task_manager: &TaskManager,
        network: &Network,
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: CachingGatewayClient,
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
        tracing::debug!("Entry gateway version: {entry_version}");
        let exit_version = selected_gateways.exit.version.clone().into();
        tracing::debug!("Exit gateway version: {exit_version}");

        // Start the auth client mixnet listener, which will listen for incoming messages from the
        // mixnet and rebroadcast them to the auth clients.
        let mixnet_listener =
            AuthClientMixnetListener::new(mixnet_client.clone(), cancel_token.child_token())
                .start();

        let auth_client = mixnet_listener
            .new_auth_client()
            .await
            .ok_or(Error::MixnetClientDisposed)?;

        let mut wg_entry_gateway_client = WgGatewayClient::new_entry(
            &data_path,
            auth_client.clone(),
            entry_auth_recipient,
            entry_version,
        );
        let mut wg_exit_gateway_client = WgGatewayClient::new_exit(
            &data_path,
            auth_client.clone(),
            exit_auth_recipient,
            exit_version,
        );

        let shutdown = task_manager.subscribe_named("bandwidth_controller");
        let (connection_data, bandwidth_controller_handle) = if let Some(data_path) =
            data_path.as_ref()
        {
            let paths = StoragePaths::new_from_dir(data_path)
                .map_err(|err| Error::SetupStoragePaths(Box::new(err)))?;
            let storage = paths
                .persistent_credential_storage()
                .await
                .map_err(|err| Error::SetupStoragePaths(Box::new(err)))?;
            let bw = BandwidthController::new(
                storage,
                network,
                wg_entry_gateway_client.light_client(),
                wg_exit_gateway_client.light_client(),
                shutdown,
                cancel_token.clone(),
            )?;
            let entry_fut = bw.get_initial_bandwidth(
                TicketType::V1WireguardEntry,
                gateway_directory_client.clone(),
                &mut wg_entry_gateway_client,
            );
            let exit_fut = bw.get_initial_bandwidth(
                TicketType::V1WireguardExit,
                gateway_directory_client.clone(),
                &mut wg_exit_gateway_client,
            );

            let (entry, exit) = Box::pin(
                cancel_token.run_until_cancelled(async { tokio::try_join!(entry_fut, exit_fut) }),
            )
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
                cancel_token.clone(),
            )?;
            let entry = bw
                .get_initial_bandwidth(
                    TicketType::V1WireguardEntry,
                    gateway_directory_client.clone(),
                    &mut wg_entry_gateway_client,
                )
                .await?;
            let exit = bw
                .get_initial_bandwidth(
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
            auth_client_mixnet_listener_handle: mixnet_listener,
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
}

struct ConnectResult {
    entry_gateway_client: WgGatewayClient,
    exit_gateway_client: WgGatewayClient,
    connection_data: ConnectionData,
    bandwidth_controller_handle: JoinHandle<()>,
    auth_client_mixnet_listener_handle: AuthClientMixnetListenerHandle,
}
