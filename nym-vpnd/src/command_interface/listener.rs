// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    path::{Path, PathBuf},
};

use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info};

use crate::service::VpnServiceCommand;

use nym_vpn_proto::{
    nym_vpnd_server::NymVpnd, ConnectRequest, ConnectResponse, DisconnectRequest,
    DisconnectResponse,
};

use super::connection_handler::CommandInterfaceConnectionHandler;

#[tonic::async_trait]
impl NymVpnd for CommandInterface {
    async fn vpn_connect(
        &self,
        request: tonic::Request<ConnectRequest>,
    ) -> Result<tonic::Response<ConnectResponse>, tonic::Status> {
        info!("Got connect request: {:?}", request);

        CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_connect()
            .await;

        info!("Returning connect response");
        Ok(tonic::Response::new(ConnectResponse { success: true }))
    }

    async fn vpn_disconnect(
        &self,
        request: tonic::Request<DisconnectRequest>,
    ) -> Result<tonic::Response<DisconnectResponse>, tonic::Status> {
        info!("Got disconnect request: {:?}", request);

        CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_disconnect()
            .await;

        info!("Returning disconnect response");
        Ok(tonic::Response::new(DisconnectResponse { success: true }))
    }
}

pub(super) struct CommandInterface {
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
    socket_path: PathBuf,
}

impl CommandInterface {
    pub(super) fn new(
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        socket_path: &Path,
    ) -> Self {
        Self {
            vpn_command_tx,
            socket_path: socket_path.to_path_buf(),
        }
    }

    fn remove_previous_socket_file(&self) {
        match fs::remove_file(&self.socket_path) {
            Ok(_) => info!(
                "Removed previous command interface socket: {:?}",
                self.socket_path
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

    #[allow(unused)]
    pub(super) async fn listen(self) {
        self.remove_previous_socket_file();
        let listener = tokio::net::UnixListener::bind(&self.socket_path).unwrap();
        info!("Command interface listening on {:?}", self.socket_path);

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone()).handle(socket);
        }
    }
}

impl Drop for CommandInterface {
    fn drop(&mut self) {
        self.remove_previous_socket_file();
    }
}
