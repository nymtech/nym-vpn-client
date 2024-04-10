// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{mpsc::UnboundedSender, oneshot},
};
use tracing::{error, info, warn};

use crate::service::{
    VpnServiceCommand, VpnServiceConnectResult, VpnServiceDisconnectResult, VpnServiceStatusResult,
};

pub(super) struct CommandInterfaceConnectionHandler {
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
}

impl CommandInterfaceConnectionHandler {
    pub(super) fn new(vpn_command_tx: UnboundedSender<VpnServiceCommand>) -> Self {
        Self { vpn_command_tx }
    }

    pub(crate) async fn handle_connect(&self) {
        info!("Starting VPN");
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Connect(tx))
            .unwrap();
        info!("Sent start command to VPN");
        info!("Waiting for response");
        match rx.await.unwrap() {
            VpnServiceConnectResult::Success => {
                info!("VPN started successfully");
            }
            VpnServiceConnectResult::Fail(err) => {
                info!("VPN failed to start: {err}");
            }
        };
    }

    pub(crate) async fn handle_disconnect(&self) {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Disconnect(tx))
            .unwrap();
        info!("Sent stop command to VPN");
        info!("Waiting for response");
        match rx.await.unwrap() {
            VpnServiceDisconnectResult::Success => {
                info!("VPN stopped successfully");
            }
            VpnServiceDisconnectResult::NotRunning => {
                info!("VPN can't stop - it's not running");
            }
            VpnServiceDisconnectResult::Fail(err) => {
                warn!("VPN failed to stop: {err}");
            }
        };
    }

    async fn handle_status(&self) -> VpnServiceStatusResult {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Status(tx))
            .unwrap();
        info!("Sent status command to VPN");
        info!("Waiting for response");
        let status = rx.await.unwrap();
        info!("VPN status: {:?}", status);
        status
    }

    pub(super) fn handle(self, mut socket: tokio::net::UnixStream) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            match socket.read(&mut buffer).await {
                Ok(0) => info!("Received 0 bytes"),
                Ok(n) => {
                    let command = std::str::from_utf8(&buffer[..n]).unwrap().trim();
                    info!("Command: Received command: {:?}", command);
                    match command {
                        "connect" => {
                            self.handle_connect().await;
                        }
                        "disconnect" => {
                            self.handle_disconnect().await;
                        }
                        "status" => {
                            let status = self.handle_status().await;
                            socket
                                .write_all(format!("{:?}", status).as_bytes())
                                .await
                                .unwrap();
                        }
                        command => info!("Unknown command: {}", command),
                    }
                }
                Err(e) => error!("Failed to read from socket; err = {:?}", e),
            }
        })
    }
}
