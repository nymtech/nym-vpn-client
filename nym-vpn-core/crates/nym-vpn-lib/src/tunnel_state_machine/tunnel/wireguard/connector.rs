// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, path::PathBuf};

use nym_config::defaults::{WG_METADATA_PORT, WG_TUN_DEVICE_IP_ADDRESS_V4};
use nym_vpn_network_config::Network;

use nym_authenticator_client::{AuthClientMixnetListener, AuthClientMixnetListenerHandle};
use nym_credentials_interface::TicketType;
use nym_gateway_directory::{AuthAddresses, CachingGatewayClient, Gateway};
use nym_sdk::mixnet::{EphemeralCredentialStorage, StoragePaths};
use nym_task::TaskManager;
use nym_wg_gateway_client::{GatewayData, WgGatewayClient};
use nym_wg_metadata_client::MetadataClient;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    bandwidth_controller::{
        BandwidthController, GATEWAY_METADATA_UPDATE_VERSION, TemporaryBandwidthClient,
    },
    mixnet::SharedMixnetClient,
    tunnel_state_machine::tunnel::{self, Error, Result, gateway_selector::SelectedGateways},
};

pub struct ConnectionData {
    pub entry: GatewayData,
    pub exit: GatewayData,
}

pub struct InterfaceIpSender {
    pub entry_tx: tokio::sync::oneshot::Sender<IpAddr>,
    pub exit_tx: tokio::sync::oneshot::Sender<IpAddr>,
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

    pub(crate) async fn connect(
        self,
        task_manager: &TaskManager,
        network: &Network,
        selected_gateways: SelectedGateways,
        data_path: Option<PathBuf>,
        cancel_token: CancellationToken,
    ) -> Result<ConnectResult> {
        Box::pin(Self::connect_inner(
            task_manager,
            network,
            self.mixnet_client.clone(),
            self.gateway_directory_client.clone(),
            selected_gateways,
            data_path,
            cancel_token,
        ))
        .await
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

        let entry_auth_version = selected_gateways.entry.version.clone().into();
        tracing::debug!("Entry gateway authenticator version: {entry_auth_version}");
        let exit_auth_version = selected_gateways.exit.version.clone().into();
        tracing::debug!("Exit gateway authenticator version: {exit_auth_version}");

        // Start the auth client mixnet listener, which will listen for incoming messages from the
        // mixnet and rebroadcast them to the auth clients.
        let mixnet_listener =
            AuthClientMixnetListener::new(mixnet_client.clone(), cancel_token.child_token())
                .start();

        let auth_mix_client = mixnet_listener
            .new_auth_client()
            .await
            .ok_or(Error::MixnetClientDisposed)?;

        let mut wg_entry_gateway_client = WgGatewayClient::new_entry(
            &data_path,
            auth_mix_client.clone(),
            entry_auth_recipient,
            entry_auth_version,
        );
        let mut wg_exit_gateway_client = WgGatewayClient::new_exit(
            &data_path,
            auth_mix_client.clone(),
            exit_auth_recipient,
            exit_auth_version,
        );

        // this shouldn't fail, verified by unit test as well
        let gateway_private_url = Url::parse(&format!(
            "http://{WG_TUN_DEVICE_IP_ADDRESS_V4}:{WG_METADATA_PORT}"
        ))
        .expect("invalid gateway private URL");

        let (entry_tx, entry_rx) = tokio::sync::oneshot::channel();
        let wg_entry_metadata_client = MetadataClient::new(
            gateway_private_url.clone(),
            selected_gateways.entry.identity(),
            entry_rx,
        );

        let (exit_tx, exit_rx) = tokio::sync::oneshot::channel();
        let wg_exit_metadata_client = MetadataClient::new(
            gateway_private_url,
            selected_gateways.exit.identity(),
            exit_rx,
        );

        let wg_entry_client = if let Some(version) = selected_gateways.entry.version
            && let Ok(version) = semver::Version::parse(&version)
            && version >= GATEWAY_METADATA_UPDATE_VERSION
        {
            tracing::debug!("Using latest metadata client for entry bandwidth controller");
            TemporaryBandwidthClient::Latest(wg_entry_metadata_client)
        } else {
            tracing::debug!("Using deprecated mixnet client for entry bandwidth controller");
            TemporaryBandwidthClient::Deprecated(wg_entry_gateway_client.light_client())
        };
        let wg_exit_client = if let Some(version) = selected_gateways.exit.version
            && let Ok(version) = semver::Version::parse(&version)
            && version >= GATEWAY_METADATA_UPDATE_VERSION
        {
            tracing::debug!("Using latest metadata client for exit bandwidth controller");
            TemporaryBandwidthClient::Latest(wg_exit_metadata_client)
        } else {
            tracing::debug!("Using deprecated mixnet client for exit bandwidth controller");
            TemporaryBandwidthClient::Deprecated(wg_exit_gateway_client.light_client())
        };

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
                wg_entry_client,
                wg_exit_client,
                shutdown,
                cancel_token.clone(),
            )?;
            let entry_fut = bw.register(
                TicketType::V1WireguardEntry,
                gateway_directory_client.clone(),
                &mut wg_entry_gateway_client,
            );
            let exit_fut = bw.register(
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
                wg_entry_client,
                wg_exit_client,
                shutdown,
                cancel_token.clone(),
            )?;
            let entry = bw
                .register(
                    TicketType::V1WireguardEntry,
                    gateway_directory_client.clone(),
                    &mut wg_entry_gateway_client,
                )
                .await?;
            let exit = bw
                .register(
                    TicketType::V1WireguardExit,
                    gateway_directory_client,
                    &mut wg_exit_gateway_client,
                )
                .await?;

            let bandwidth_controller_handle = tokio::spawn(bw.run());

            (ConnectionData { entry, exit }, bandwidth_controller_handle)
        };

        Ok(ConnectResult {
            entry_gateway_client: wg_entry_gateway_client,
            exit_gateway_client: wg_exit_gateway_client,
            connection_data,
            bandwidth_controller_handle,
            auth_client_mixnet_listener_handle: mixnet_listener,
            interface_ip_sender: InterfaceIpSender { entry_tx, exit_tx },
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

pub(crate) struct ConnectResult {
    pub(crate) entry_gateway_client: WgGatewayClient,
    pub(crate) exit_gateway_client: WgGatewayClient,
    pub(crate) connection_data: ConnectionData,
    pub(crate) bandwidth_controller_handle: JoinHandle<()>,
    pub(crate) auth_client_mixnet_listener_handle: AuthClientMixnetListenerHandle,
    pub(crate) interface_ip_sender: InterfaceIpSender,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url() {
        assert!(
            Url::parse(&format!(
                "http://{WG_TUN_DEVICE_IP_ADDRESS_V4}:{WG_METADATA_PORT}"
            ))
            .is_ok()
        );
    }
}
