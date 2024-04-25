// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use nym_vpn_lib::{
    gateway_directory::{EntryPoint, ExitPoint},
    NodeIdentity, Recipient,
};
use nym_vpn_proto::{
    nym_vpnd_server::NymVpnd, ConnectRequest, ConnectResponse, ConnectionStatus, DisconnectRequest,
    DisconnectResponse, Error as ProtoError, ImportUserCredentialRequest,
    ImportUserCredentialResponse, StatusRequest, StatusResponse,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info};

use super::connection_handler::CommandInterfaceConnectionHandler;
use crate::service::{VpnServiceCommand, VpnServiceStatusResult};

enum ListenerType {
    Path(PathBuf),
    Uri(#[allow(unused)] SocketAddr),
}

pub(super) struct CommandInterface {
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    listener: ListenerType,
}

impl CommandInterface {
    pub(super) fn new_with_path(
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        socket_path: &Path,
    ) -> Self {
        Self {
            vpn_command_tx,
            listener: ListenerType::Path(socket_path.to_path_buf()),
        }
    }

    pub(super) fn new_with_uri(
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        uri: SocketAddr,
    ) -> Self {
        Self {
            vpn_command_tx,
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
    async fn vpn_connect(
        &self,
        request: tonic::Request<ConnectRequest>,
    ) -> Result<tonic::Response<ConnectResponse>, tonic::Status> {
        info!("Got connect request: {:?}", request);

        let connect_request = request.into_inner();

        let entry = if let Some(entry) = connect_request.entry {
            if let Some(entry_node_enum) = entry.entry_node_enum {
                match entry_node_enum {
                    nym_vpn_proto::entry_node::EntryNodeEnum::Location(location) => {
                        info!(
                            "Connecting to entry node in country: {:?}",
                            location.two_letter_iso_country_code
                        );
                        Some(EntryPoint::Location {
                            location: location.two_letter_iso_country_code.to_string(),
                        })
                    }
                    nym_vpn_proto::entry_node::EntryNodeEnum::Gateway(gateway) => {
                        info!("Connecting to entry node with gateway id: {:?}", gateway.id);
                        let identity =
                            NodeIdentity::from_base58_string(&gateway.id).map_err(|err| {
                                error!("Failed to parse gateway id: {:?}", err);
                                tonic::Status::invalid_argument("Invalid gateway id")
                            })?;
                        Some(EntryPoint::Gateway { identity })
                    }
                    nym_vpn_proto::entry_node::EntryNodeEnum::RandomLowLatency(_) => {
                        info!("Connecting to low latency entry node");
                        Some(EntryPoint::RandomLowLatency)
                    }
                    nym_vpn_proto::entry_node::EntryNodeEnum::Random(_) => {
                        info!("Connecting to random entry node");
                        Some(EntryPoint::Random)
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let exit = if let Some(exit) = connect_request.exit {
            if let Some(exit_node_enum) = exit.exit_node_enum {
                match exit_node_enum {
                    nym_vpn_proto::exit_node::ExitNodeEnum::Address(address) => {
                        info!(
                            "Connecting to exit node at address: {:?}",
                            address.nym_address
                        );
                        let address = Recipient::try_from_base58_string(address.nym_address)
                            .map_err(|err| {
                                error!("Failed to parse exit node address: {:?}", err);
                                tonic::Status::invalid_argument("Invalid exit node address")
                            })?;
                        Some(ExitPoint::Address { address })
                    }
                    nym_vpn_proto::exit_node::ExitNodeEnum::Gateway(gateway) => {
                        info!("Connecting to exit node with gateway id: {:?}", gateway.id);
                        let identity =
                            NodeIdentity::from_base58_string(&gateway.id).map_err(|err| {
                                error!("Failed to parse gateway id: {:?}", err);
                                tonic::Status::invalid_argument("Invalid gateway id")
                            })?;
                        Some(ExitPoint::Gateway { identity })
                    }
                    nym_vpn_proto::exit_node::ExitNodeEnum::Location(location) => {
                        info!(
                            "Connecting to exit node in country: {:?}",
                            location.two_letter_iso_country_code
                        );
                        Some(ExitPoint::Location {
                            location: location.two_letter_iso_country_code.to_string(),
                        })
                    }
                    nym_vpn_proto::exit_node::ExitNodeEnum::Random(_) => {
                        info!("Connecting to low latency exit node");
                        Some(ExitPoint::Random)
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_connect(entry, exit)
            .await;

        info!("Returning connect response");
        Ok(tonic::Response::new(ConnectResponse {
            success: status.is_success(),
        }))
    }

    async fn vpn_disconnect(
        &self,
        request: tonic::Request<DisconnectRequest>,
    ) -> Result<tonic::Response<DisconnectResponse>, tonic::Status> {
        info!("Got disconnect request: {:?}", request);

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_disconnect()
            .await;

        info!("Returning disconnect response");
        Ok(tonic::Response::new(DisconnectResponse {
            success: status.is_success(),
        }))
    }

    async fn vpn_status(
        &self,
        request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        info!("Got status request: {:?}", request);

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_status()
            .await;

        let error = match status {
            VpnServiceStatusResult::NotConnected => None,
            VpnServiceStatusResult::Connecting => None,
            VpnServiceStatusResult::Connected => None,
            VpnServiceStatusResult::Disconnecting => None,
            VpnServiceStatusResult::ConnectionFailed(ref reason) => Some(reason.clone()),
        }
        .map(|reason| ProtoError { message: reason });

        info!("Returning status response");
        Ok(tonic::Response::new(StatusResponse {
            status: ConnectionStatus::from(status) as i32,
            error,
        }))
    }

    async fn import_user_credential(
        &self,
        request: tonic::Request<ImportUserCredentialRequest>,
    ) -> Result<tonic::Response<ImportUserCredentialResponse>, tonic::Status> {
        info!("Got import credential request");

        let credential = request.into_inner().credential;

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_import_credential(credential)
            .await;

        info!("Returning import credential response");
        Ok(tonic::Response::new(ImportUserCredentialResponse {
            success: status.is_success(),
        }))
    }
}

impl From<VpnServiceStatusResult> for ConnectionStatus {
    fn from(status: VpnServiceStatusResult) -> Self {
        match status {
            VpnServiceStatusResult::NotConnected => ConnectionStatus::NotConnected,
            VpnServiceStatusResult::Connecting => ConnectionStatus::Connecting,
            VpnServiceStatusResult::Connected => ConnectionStatus::Connected,
            VpnServiceStatusResult::Disconnecting => ConnectionStatus::Disconnecting,
            VpnServiceStatusResult::ConnectionFailed(_reason) => ConnectionStatus::ConnectionFailed,
        }
    }
}
