// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    path::{Path, PathBuf},
};

use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info};

use crate::service::VpnServiceCommand;

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

    pub(super) async fn listen(self) {
        self.remove_previous_socket_file();
        let listener = tokio::net::UnixListener::bind(&self.socket_path).unwrap();
        info!("Command interface listening on {:?}", self.socket_path);

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            super::connection_handler::CommandInterfaceConnectionHandler::new(
                self.vpn_command_tx.clone(),
            )
            .handle(socket);
        }
    }
}

impl Drop for CommandInterface {
    fn drop(&mut self) {
        self.remove_previous_socket_file();
    }
}
