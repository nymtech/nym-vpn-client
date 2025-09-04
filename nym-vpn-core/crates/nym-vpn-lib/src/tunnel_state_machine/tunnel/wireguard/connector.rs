// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_network_config::Network;

use nym_authenticator_client::{
    AuthClientMixnetListener, AuthClientMixnetListenerHandle, AuthenticatorVersion,
};
use nym_gateway_directory::{CachingGatewayClient, Gateway, Recipient};
use nym_sdk::mixnet::{EphemeralCredentialStorage, StoragePaths};
use nym_task::TaskManager;
use nym_wg_gateway_client::{GatewayData, WgGatewayClient};
use nym_wg_metadata_client::TunUpEvent;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    bandwidth_controller::{BandwidthController, get_nyxd_client},
    mixnet::SharedMixnetClient,
    tunnel_state_machine::tunnel::{
        Error, Result, gateway_selector::SelectedGateways,
        wireguard::connected_tunnel::ConnectedTunnel,
    },
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
    ) -> Result<(ConnectedTunnel, TunUpEvent)> {
        let (connect_result, tun_up) = Box::pin(Self::connect_inner(
            task_manager,
            network,
            self.mixnet_client.clone(),
            self.gateway_directory_client.clone(),
            selected_gateways,
            data_path,
            cancel_token,
        ))
        .await?;
        Ok((
            ConnectedTunnel::new(
                connect_result.entry_gateway_client,
                connect_result.exit_gateway_client,
                connect_result.connection_data,
                connect_result.bandwidth_controller_handle,
                connect_result.auth_client_mixnet_listener_handle,
            ),
            tun_up,
        ))
    }

    fn get_recipient_and_version(gateway: &Gateway) -> Result<(Recipient, AuthenticatorVersion)> {
        let Some(auth_recipient) = gateway
            .authenticator_address
            .ok_or(Error::AuthenticatorAddressNotFound)?
            .0
        else {
            return Err(Error::AuthenticationNotPossible(
                gateway.identity.to_string(),
            ));
        };
        let auth_version = gateway.version.clone().into();

        Ok((auth_recipient, auth_version))
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
    ) -> Result<(ConnectResult, TunUpEvent)> {
        // Start the auth client mixnet listener, which will listen for incoming messages from the
        // mixnet and rebroadcast them to the auth clients.
        let mixnet_listener =
            AuthClientMixnetListener::new(mixnet_client.clone(), cancel_token.child_token())
                .start();

        let auth_mix_client = mixnet_listener
            .new_auth_client()
            .await
            .ok_or(Error::MixnetClientDisposed)?;

        let (entry_recipient, entry_version) =
            Self::get_recipient_and_version(&selected_gateways.entry)?;
        let mut wg_entry_gateway_client = WgGatewayClient::new_entry(
            &data_path,
            auth_mix_client.clone(),
            entry_recipient,
            entry_version,
        );

        let (exit_recipient, exit_version) =
            Self::get_recipient_and_version(&selected_gateways.exit)?;
        let mut wg_exit_gateway_client =
            WgGatewayClient::new_exit(&data_path, auth_mix_client, exit_recipient, exit_version);

        let client = get_nyxd_client(network)?;
        let (entry_signal_tx, entry_signal_rx) = tokio::sync::oneshot::channel();
        let (exit_signal_tx, exit_signal_rx) = tokio::sync::oneshot::channel();

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

            let controller = nym_bandwidth_controller::BandwidthController::new(storage, client);

            let (bw, connection_data) = BandwidthController::register_and_create(
                controller,
                &gateway_directory_client,
                selected_gateways,
                &mut wg_entry_gateway_client,
                &mut wg_exit_gateway_client,
                entry_signal_rx,
                exit_signal_rx,
                shutdown,
                cancel_token,
            )
            .await?;
            let bandwidth_controller_handle = tokio::spawn(bw.run());
            (connection_data, bandwidth_controller_handle)
        } else {
            let storage = EphemeralCredentialStorage::default();
            let controller = nym_bandwidth_controller::BandwidthController::new(storage, client);
            let (bw, connection_data) = BandwidthController::register_and_create(
                controller,
                &gateway_directory_client,
                selected_gateways,
                &mut wg_entry_gateway_client,
                &mut wg_exit_gateway_client,
                entry_signal_rx,
                exit_signal_rx,
                shutdown,
                cancel_token,
            )
            .await?;
            let bandwidth_controller_handle = tokio::spawn(bw.run());
            (connection_data, bandwidth_controller_handle)
        };

        Ok((
            ConnectResult {
                entry_gateway_client: wg_entry_gateway_client,
                exit_gateway_client: wg_exit_gateway_client,
                connection_data,
                bandwidth_controller_handle,
                auth_client_mixnet_listener_handle: mixnet_listener,
            },
            TunUpEvent {
                entry: entry_signal_tx,
                exit: exit_signal_tx,
            },
        ))
    }
}

struct ConnectResult {
    pub(crate) entry_gateway_client: WgGatewayClient,
    pub(crate) exit_gateway_client: WgGatewayClient,
    pub(crate) connection_data: ConnectionData,
    pub(crate) bandwidth_controller_handle: JoinHandle<()>,
    pub(crate) auth_client_mixnet_listener_handle: AuthClientMixnetListenerHandle,
}
