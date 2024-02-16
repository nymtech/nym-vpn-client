use std::path::Path;

use tokio::{
    io::AsyncReadExt,
    sync::{
        mpsc::{Receiver, Sender},
        oneshot,
    },
};

#[derive(Debug)]
enum VpnCommand {
    Start(oneshot::Sender<VpnCommandResult>),
    Stop,
    Restart,
}

#[derive(Debug)]
enum VpnCommandResult {
    Started,
    Stopped,
    Restarted,
}

fn start_command_handler() -> (std::thread::JoinHandle<()>, Receiver<VpnCommand>) {
    // Channel to send commands to the vpn handler
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::channel(32);

    let handle = std::thread::spawn(move || {
        let command_rt = tokio::runtime::Runtime::new().unwrap();
        command_rt.block_on(async {
            listen_for_commands(Path::new("/tmp/nym-vpn.socket"), vpn_command_tx).await;
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
                        "start" => {
                            println!("Starting VPN");
                            let (tx, rx) = oneshot::channel();
                            vpn_command_tx.send(VpnCommand::Start(tx)).await.unwrap();
                            println!("Sent start command to VPN");
                            println!("Waiting for response");
                            match rx.await.unwrap() {
                                VpnCommandResult::Started => {
                                    println!("VPN started successfully");
                                }
                                _ => println!("Failed to start VPN"),
                            }
                        }
                        "stop" => {
                            vpn_command_tx.send(VpnCommand::Stop).await.unwrap();
                        }
                        "restart" => {
                            vpn_command_tx.send(VpnCommand::Restart).await.unwrap();
                        }
                        command => println!("Unknown command: {}", command),
                    }
                }
                Err(e) => println!("Failed to read from socket; err = {:?}", e),
            }
        });
    }
}

fn start_vpn_handler(mut vpn_command_rx: Receiver<VpnCommand>) -> std::thread::JoinHandle<()> {
    println!("Starting VPN handler");
    std::thread::spawn(move || {
        let vpn_rt = tokio::runtime::Runtime::new().unwrap();
        vpn_rt.block_on(async {
            // Listen to the command channel
            println!("VPN: Listening for commands");
            while let Some(command) = vpn_command_rx.recv().await {
                println!("VPN: Received command: {:?}", command);
                match command {
                    VpnCommand::Start(tx) => {
                        // Start the VPN
                        tx.send(VpnCommandResult::Started).unwrap();
                    }
                    VpnCommand::Stop => {
                        // Stop the VPN
                    }
                    VpnCommand::Restart => {
                        // Restart the VPN
                    }
                }
            }
        });
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("main: starting command handler");
    let (command_handle, vpn_command_rx) = start_command_handler();

    println!("main: starting VPN handler");
    let vpn_handle = start_vpn_handler(vpn_command_rx);

    command_handle.join().unwrap();
    vpn_handle.join().unwrap();

    Ok(())
}
