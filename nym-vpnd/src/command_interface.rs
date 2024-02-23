use std::path::Path;

use tokio::io::AsyncWriteExt;
use tokio::{
    io::AsyncReadExt,
    sync::{
        mpsc::{Receiver, Sender},
        oneshot,
    },
};

#[derive(Debug)]
pub enum VpnCommand {
    Connect(oneshot::Sender<VpnConnectResult>),
    Disconnect(oneshot::Sender<VpnDisconnectResult>),
    Status(oneshot::Sender<VpnStatusResult>),
}

#[derive(Debug)]
pub enum VpnConnectResult {
    Success,
    #[allow(unused)]
    Fail(String),
}

#[derive(Debug)]
pub enum VpnDisconnectResult {
    Success,
    NotRunning,
    #[allow(unused)]
    Fail(String),
}

#[derive(Copy, Clone, Debug)]
pub enum VpnStatusResult {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
}

pub fn start_command_interface() -> (std::thread::JoinHandle<()>, Receiver<VpnCommand>) {
    // Channel to send commands to the vpn handler
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::channel(32);

    let handle = std::thread::spawn(move || {
        let command_rt = tokio::runtime::Runtime::new().unwrap();
        command_rt.block_on(async {
            listen_for_commands(Path::new("/var/run/nym-vpn.socket"), vpn_command_tx).await;
        });
    });

    (handle, vpn_command_rx)
}

async fn listen_for_commands(socket_path: &Path, vpn_command_tx: Sender<VpnCommand>) {
    let listener = tokio::net::UnixListener::bind(socket_path).unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let vpn_command_tx = vpn_command_tx.clone();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            match socket.read(&mut buffer).await {
                Ok(0) => println!("Received 0 bytes"),
                Ok(n) => {
                    let command = std::str::from_utf8(&buffer[..n]).unwrap().trim();
                    println!("Command: Received command: {:?}", command);
                    match command {
                        "connect" => {
                            println!("Starting VPN");
                            let (tx, rx) = oneshot::channel();
                            vpn_command_tx.send(VpnCommand::Connect(tx)).await.unwrap();
                            println!("Sent start command to VPN");
                            println!("Waiting for response");
                            match rx.await.unwrap() {
                                VpnConnectResult::Success => {
                                    println!("VPN started successfully");
                                }
                                VpnConnectResult::Fail(err) => {
                                    println!("VPN failed to start: {err}");
                                }
                            }
                        }
                        "disconnect" => {
                            let (tx, rx) = oneshot::channel();
                            vpn_command_tx
                                .send(VpnCommand::Disconnect(tx))
                                .await
                                .unwrap();
                            println!("Sent stop command to VPN");
                            println!("Waiting for response");
                            match rx.await.unwrap() {
                                VpnDisconnectResult::Success => {
                                    println!("VPN stopped successfully");
                                }
                                VpnDisconnectResult::NotRunning => {
                                    println!("VPN can't stop - it's not running");
                                }
                                VpnDisconnectResult::Fail(err) => {
                                    println!("VPN failed to stop: {err}");
                                }
                            }
                        }
                        "status" => {
                            let (tx, rx) = oneshot::channel();
                            vpn_command_tx.send(VpnCommand::Status(tx)).await.unwrap();
                            println!("Sent status command to VPN");
                            println!("Waiting for response");
                            let status = rx.await.unwrap();
                            println!("VPN status: {:?}", status);
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
        });
    }
}
