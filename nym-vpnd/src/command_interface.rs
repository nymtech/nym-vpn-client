use std::path::Path;

use tokio::io::AsyncWriteExt;
use tokio::{
    io::AsyncReadExt,
    sync::{
        mpsc::{Receiver, Sender},
        oneshot,
    },
};

use crate::service::{
    VpnServiceCommand, VpnServiceConnectResult, VpnServiceDisconnectResult, VpnServiceStatusResult,
};

pub fn start_command_interface() -> (std::thread::JoinHandle<()>, Receiver<VpnServiceCommand>) {
    // Channel to send commands to the vpn service
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::channel(32);

    let handle = std::thread::spawn(move || {
        let command_rt = tokio::runtime::Runtime::new().unwrap();
        command_rt.block_on(async {
            CommandInterface::new(vpn_command_tx)
                .listen(Path::new("/var/run/nym-vpn.socket"))
                .await
        });
    });

    (handle, vpn_command_rx)
}

struct CommandInterface {
    vpn_command_tx: Sender<VpnServiceCommand>,
}

impl CommandInterface {
    fn new(vpn_command_tx: Sender<VpnServiceCommand>) -> Self {
        Self { vpn_command_tx }
    }

    async fn listen(self, socket_path: &Path) {
        let listener = tokio::net::UnixListener::bind(socket_path).unwrap();

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone()).handle(socket);
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
        println!("Starting VPN");
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Connect(tx))
            .await
            .unwrap();
        println!("Sent start command to VPN");
        println!("Waiting for response");
        match rx.await.unwrap() {
            VpnServiceConnectResult::Success => {
                println!("VPN started successfully");
            }
            VpnServiceConnectResult::Fail(err) => {
                println!("VPN failed to start: {err}");
            }
        };
    }

    async fn handle_disconnect(&self) {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Disconnect(tx))
            .await
            .unwrap();
        println!("Sent stop command to VPN");
        println!("Waiting for response");
        match rx.await.unwrap() {
            VpnServiceDisconnectResult::Success => {
                println!("VPN stopped successfully");
            }
            VpnServiceDisconnectResult::NotRunning => {
                println!("VPN can't stop - it's not running");
            }
            VpnServiceDisconnectResult::Fail(err) => {
                println!("VPN failed to stop: {err}");
            }
        };
    }

    async fn handle_status(&self) -> VpnServiceStatusResult {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Status(tx))
            .await
            .unwrap();
        println!("Sent status command to VPN");
        println!("Waiting for response");
        let status = rx.await.unwrap();
        println!("VPN status: {:?}", status);
        status
    }

    fn handle(self, mut socket: tokio::net::UnixStream) -> tokio::task::JoinHandle<()> {
        // let vpn_command_tx = self.vpn_command_tx.clone();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            match socket.read(&mut buffer).await {
                Ok(0) => println!("Received 0 bytes"),
                Ok(n) => {
                    let command = std::str::from_utf8(&buffer[..n]).unwrap().trim();
                    println!("Command: Received command: {:?}", command);
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
                        command => println!("Unknown command: {}", command),
                    }
                }
                Err(e) => println!("Failed to read from socket; err = {:?}", e),
            }
        })
    }
}
