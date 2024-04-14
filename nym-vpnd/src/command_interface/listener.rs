// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use nym_vpn_proto::{
    nym_vpnd_server::NymVpnd, ConnectRequest, ConnectResponse, ConnectionStatus, DisconnectRequest,
    DisconnectResponse, StatusRequest, StatusResponse,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info};

use super::connection_handler::CommandInterfaceConnectionHandler;
use crate::service::{VpnServiceCommand, VpnServiceConnectResult, VpnServiceStatusResult};

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

    fn remove_previous_socket_file(&self) {
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

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_connect()
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

        info!("Returning status response");
        Ok(tonic::Response::new(StatusResponse {
            status: ConnectionStatus::from(status) as i32,
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
        }
    }
}
