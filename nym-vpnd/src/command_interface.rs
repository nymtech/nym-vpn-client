use std::path::{Path, PathBuf};

use nym_task::TaskManager;
use tokio::io::AsyncWriteExt;
use tokio::{
    io::AsyncReadExt,
    sync::{
        mpsc::{Receiver, Sender},
        oneshot,
    },
};
use tracing::{error, info, warn};

use crate::service::{
    VpnServiceCommand, VpnServiceConnectResult, VpnServiceDisconnectResult, VpnServiceStatusResult,
};

pub fn start_command_interface(mut task_manager: TaskManager) -> (std::thread::JoinHandle<()>, Receiver<VpnServiceCommand>) {
    // Channel to send commands to the vpn service
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::channel(32);

    let socket_path = Path::new("/var/run/nym-vpn.socket");

    let handle = std::thread::spawn(move || {
        let command_rt = tokio::runtime::Runtime::new().unwrap();
        command_rt.block_on(async {
            // Spawn command interface
            tokio::task::spawn(async {
                CommandInterface::new(vpn_command_tx, socket_path)
                    .listen()
                    .await
            });

            // Wait for interrupt
            // Send shutdown signal to all tasks
            // Wait for all tasks to finish
            let _ = task_manager.catch_interrupt().await;

            info!("Command interface exiting");
        });
    });

    (handle, vpn_command_rx)
}

struct CommandInterface {
    vpn_command_tx: Sender<VpnServiceCommand>,
    socket_path: PathBuf,
}

impl CommandInterface {
    fn new(vpn_command_tx: Sender<VpnServiceCommand>, socket_path: &Path) -> Self {
        Self {
            vpn_command_tx,
            socket_path: socket_path.to_path_buf(),
        }
    }

    async fn listen(self) {
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
            CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone()).handle(socket);
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

struct CommandInterfaceConnectionHandler {
    vpn_command_tx: Sender<VpnServiceCommand>,
}

impl CommandInterfaceConnectionHandler {
    fn new(vpn_command_tx: Sender<VpnServiceCommand>) -> Self {
        Self { vpn_command_tx }
    }

    async fn handle_connect(&self) {
        info!("Starting VPN");
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Connect(tx))
            .await
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

    async fn handle_disconnect(&self) {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Disconnect(tx))
            .await
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
            .await
            .unwrap();
        info!("Sent status command to VPN");
        info!("Waiting for response");
        let status = rx.await.unwrap();
        info!("VPN status: {:?}", status);
        status
    }

    fn handle(self, mut socket: tokio::net::UnixStream) -> tokio::task::JoinHandle<()> {
        // let vpn_command_tx = self.vpn_command_tx.clone();
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
