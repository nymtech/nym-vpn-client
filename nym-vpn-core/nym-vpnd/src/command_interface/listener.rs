// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::SystemTime;
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use futures::{stream::BoxStream, StreamExt};
use nym_vpn_proto::{
    nym_vpnd_server::NymVpnd, ConnectRequest, ConnectResponse, ConnectionStateChange,
    ConnectionStatusUpdate, DisconnectRequest, DisconnectResponse, Empty,
    ImportUserCredentialRequest, ImportUserCredentialResponse, StatusRequest, StatusResponse,
};
use nym_vpn_proto::{
    InfoRequest, InfoResponse, ListEntryGatewaysRequest, ListEntryGatewaysResponse,
    ListExitGatewaysRequest, ListExitGatewaysResponse,
};
use prost_types::Timestamp;
use tokio::sync::{broadcast, mpsc::UnboundedSender};
use tracing::{error, info};

use super::{
    connection_handler::CommandInterfaceConnectionHandler,
    error::CommandInterfaceError,
    helpers::{parse_entry_point, parse_exit_point},
    status_broadcaster::ConnectionStatusBroadcaster,
};
use crate::service::{
    ConnectOptions, VpnServiceCommand, VpnServiceConnectResult, VpnServiceStateChange,
};

enum ListenerType {
    Path(PathBuf),
    Uri(#[allow(unused)] SocketAddr),
}

pub(super) struct CommandInterface {
    // Listen to state changes from the VPN service
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,

    // Send commands to the VPN service
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,

    // Broadcast connection status updates to our API endpoint listeners
    status_tx: tokio::sync::broadcast::Sender<ConnectionStatusUpdate>,

    listener: ListenerType,
}

impl CommandInterface {
    pub(super) fn new_with_path(
        vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        socket_path: &Path,
    ) -> Self {
        Self {
            vpn_state_changes_rx,
            vpn_command_tx,
            status_tx: tokio::sync::broadcast::channel(10).0,
            listener: ListenerType::Path(socket_path.to_path_buf()),
        }
    }

    pub(super) fn new_with_uri(
        vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        uri: SocketAddr,
    ) -> Self {
        Self {
            vpn_state_changes_rx,
            vpn_command_tx,
            status_tx: tokio::sync::broadcast::channel(10).0,
            listener: ListenerType::Uri(uri),
        }
    }

    pub(super) fn remove_previous_socket_file(&self) {
        if let ListenerType::Path(ref socket_path) = self.listener {
            match fs::remove_file(socket_path) {
                Ok(_) => info!(
                    "Removed previous command interface socket: {:?}",
                    socket_path
                ),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => {
                    error!(
                        "Failed to remove previous command interface socket: {:?}",
                        err
                    );
                }
            }
        }
    }
}

impl Drop for CommandInterface {
    fn drop(&mut self) {
        self.remove_previous_socket_file();
    }
}

#[tonic::async_trait]
impl NymVpnd for CommandInterface {
    async fn info(
        &self,
        request: tonic::Request<InfoRequest>,
    ) -> Result<tonic::Response<InfoResponse>, tonic::Status> {
        info!("Got info request: {:?}", request);

        let info = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_info()
            .await;

        let response = InfoResponse::from(info);
        info!("Returning info response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_connect(
        &self,
        request: tonic::Request<ConnectRequest>,
    ) -> Result<tonic::Response<ConnectResponse>, tonic::Status> {
        info!("Got connect request: {:?}", request);

        let connect_request = request.into_inner();

        let entry = connect_request
            .entry
            .clone()
            .and_then(|e| e.entry_node_enum)
            .map(parse_entry_point)
            .transpose()?;

        let exit = connect_request
            .exit
            .clone()
            .and_then(|e| e.exit_node_enum)
            .map(parse_exit_point)
            .transpose()?;

        let options = ConnectOptions::try_from(connect_request).map_err(|err| {
            error!("Failed to parse connect options: {:?}", err);
            tonic::Status::invalid_argument("Invalid connect options")
        })?;

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_connect(entry, exit, options)
            .await;

        let success = status.is_success();

        // After connecting we start a task that listens for status updates and broadcasts them for
        // listeners to the connection status stream.
        if let VpnServiceConnectResult::Success(connect_handle) = status {
            ConnectionStatusBroadcaster::new(
                self.status_tx.clone(),
                connect_handle.listener_vpn_status_rx,
            )
            .start();
        }

        let response = ConnectResponse { success };
        info!("Returning connect response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_disconnect(
        &self,
        request: tonic::Request<DisconnectRequest>,
    ) -> Result<tonic::Response<DisconnectResponse>, tonic::Status> {
        info!("Got disconnect request: {:?}", request);

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_disconnect()
            .await;

        let response = DisconnectResponse {
            success: status.is_success(),
        };
        info!("Returning disconnect response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_status(
        &self,
        request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        info!("Got status request: {:?}", request);

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_status()
            .await;

        let response = StatusResponse::from(status);
        info!("Returning status response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn import_user_credential(
        &self,
        request: tonic::Request<ImportUserCredentialRequest>,
    ) -> Result<tonic::Response<ImportUserCredentialResponse>, tonic::Status> {
        info!("Got import credential request");

        let credential = request.into_inner().credential;

        let response = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_import_credential(credential)
            .await;

        let response = match response {
            Ok(time) => ImportUserCredentialResponse {
                success: true,
                error: None,
                expiry: time.map(|t| Timestamp::from(SystemTime::from(t))),
            },
            Err(err) => ImportUserCredentialResponse {
                success: false,
                error: Some(err.into()),
                expiry: None,
            },
        };
        info!("Returning import credential response: {:?}", response);

        Ok(tonic::Response::new(response))
    }

    type ListenToConnectionStatusStream =
        BoxStream<'static, Result<ConnectionStatusUpdate, tonic::Status>>;

    async fn listen_to_connection_status(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Self::ListenToConnectionStatusStream>, tonic::Status> {
        info!("Got connection status stream request: {request:?}");
        let rx = self.status_tx.subscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|status| {
            status.map_err(|err| {
                error!("Failed to receive connection status update: {:?}", err);
                tonic::Status::internal("Failed to receive connection status update")
            })
        });
        Ok(tonic::Response::new(
            Box::pin(stream) as Self::ListenToConnectionStatusStream
        ))
    }

    type ListenToConnectionStateChangesStream =
        BoxStream<'static, Result<ConnectionStateChange, tonic::Status>>;

    async fn listen_to_connection_state_changes(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Self::ListenToConnectionStateChangesStream>, tonic::Status> {
        info!("Got connection status stream request: {request:?}");
        let rx = self.vpn_state_changes_rx.resubscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|status| {
            status.map(ConnectionStateChange::from).map_err(|err| {
                error!("Failed to receive connection state change: {:?}", err);
                tonic::Status::internal("Failed to receive connection state change")
            })
        });
        Ok(tonic::Response::new(
            Box::pin(stream) as Self::ListenToConnectionStateChangesStream
        ))
    }

    async fn list_entry_gateways(
        &self,
        request: tonic::Request<ListEntryGatewaysRequest>,
    ) -> Result<tonic::Response<ListEntryGatewaysResponse>, tonic::Status> {
        info!("Got list entry gateways request: {:?}", request);

        let entry_gateways = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_list_entry_gateways()
            .await
            .map_err(|err| {
                let msg = format!("Failed to list entry gateways: {:?}", err);
                error!(msg);
                tonic::Status::internal(msg)
            })?;

        let response = ListEntryGatewaysResponse {
            gateways: entry_gateways
                .into_iter()
                .map(nym_vpn_proto::EntryGateway::from)
                .collect(),
        };

        info!(
            "Returning list entry gateways response: {} entries",
            response.gateways.len()
        );
        Ok(tonic::Response::new(response))
    }

    async fn list_exit_gateways(
        &self,
        request: tonic::Request<ListExitGatewaysRequest>,
    ) -> Result<tonic::Response<ListExitGatewaysResponse>, tonic::Status> {
        info!("Got list exit gateways request: {:?}", request);

        let exit_gateways = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_list_exit_gateways()
            .await
            .map_err(|err| {
                let msg = format!("Failed to list exit gateways: {:?}", err);
                error!(msg);
                tonic::Status::internal(msg)
            })?;

        let response = ListExitGatewaysResponse {
            gateways: exit_gateways
                .into_iter()
                .map(nym_vpn_proto::ExitGateway::from)
                .collect(),
        };

        info!(
            "Returning list exit gateways response: {} entries",
            response.gateways.len()
        );
        Ok(tonic::Response::new(response))
    }
}

impl TryFrom<ConnectRequest> for ConnectOptions {
    type Error = CommandInterfaceError;

    fn try_from(request: ConnectRequest) -> Result<Self, Self::Error> {
        // Parse the inner DNS IP address if it exists, but make sure to keep the outer Option.
        let dns = request
            .dns
            .map(|dns| {
                dns.ip
                    .parse()
                    .map_err(|err| CommandInterfaceError::FailedToParseDnsIp {
                        ip: dns.ip.clone(),
                        source: err,
                    })
            })
            .transpose()?;

        Ok(ConnectOptions {
            dns,
            disable_routing: request.disable_routing,
            enable_two_hop: request.enable_two_hop,
            enable_poisson_rate: request.enable_poisson_rate,
            disable_background_cover_traffic: request.disable_background_cover_traffic,
            enable_credentials_mode: request.enable_credentials_mode,
        })
    }
}
