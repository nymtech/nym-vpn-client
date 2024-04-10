// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use tokio::sync::mpsc::Sender;
use tracing::{error, info};

use crate::service::VpnServiceCommand;

pub(super) struct CommandInterface {
    vpn_command_tx: Sender<VpnServiceCommand>,
    socket_path: PathBuf,
}

impl CommandInterface {
    pub(super) fn new(vpn_command_tx: Sender<VpnServiceCommand>, socket_path: &Path) -> Self {
        Self {
            vpn_command_tx,
            socket_path: socket_path.to_path_buf(),
        }
    }

    pub(super) async fn listen(self) {
        // Remove any previous file just in case
        if let Err(err) = std::fs::remove_file(&self.socket_path) {
            info!(
                "Failed to remove previous command interface socket: {:?}",
                err
            );
        }

        let listener = tokio::net::UnixListener::bind(&self.socket_path).unwrap();

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
        info!("Removing command interface socket: {:?}", self.socket_path);
        match std::fs::remove_file(&self.socket_path) {
            Ok(_) => info!("Removed command interface socket: {:?}", self.socket_path),
            Err(e) => error!("Failed to remove command interface socket: {:?}", e),
        }
    }
}
