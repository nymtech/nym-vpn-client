use std::path::Path;
use std::sync::Arc;

use futures::SinkExt;
use futures::{channel::mpsc::UnboundedSender, StreamExt};
use tokio::{
    io::AsyncReadExt,
    sync::{
        mpsc::{Receiver, Sender},
        oneshot,
    },
};

#[derive(Debug)]
enum VpnCommand {
    Start(oneshot::Sender<VpnStartResult>),
    Stop(oneshot::Sender<VpnStopResult>),
    Restart,
    Status(oneshot::Sender<VpnStatusResult>),
}

#[derive(Debug)]
enum VpnStartResult {
    Success,
    Fail(String),
}

#[derive(Debug)]
enum VpnStopResult {
    Success,
    NotRunning,
    Fail(String),
}

#[derive(Debug)]
enum VpnStatusResult {
    Running,
    NotRunning,
}

fn start_command_interface() -> (std::thread::JoinHandle<()>, Receiver<VpnCommand>) {
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
                        "start" => {
                            println!("Starting VPN");
                            let (tx, rx) = oneshot::channel();
                            vpn_command_tx.send(VpnCommand::Start(tx)).await.unwrap();
                            println!("Sent start command to VPN");
                            println!("Waiting for response");
                            match rx.await.unwrap() {
                                VpnStartResult::Success => {
                                    println!("VPN started successfully");
                                }
                                VpnStartResult::Fail(err) => {
                                    println!("VPN failed to start: {err}");
                                }
                            }
                        }
                        "stop" => {
                            let (tx, rx) = oneshot::channel();
                            vpn_command_tx.send(VpnCommand::Stop(tx)).await.unwrap();
                            println!("Sent stop command to VPN");
                            println!("Waiting for response");
                            match rx.await.unwrap() {
                                VpnStopResult::Success => {
                                    println!("VPN stopped successfully");
                                }
                                VpnStopResult::NotRunning => {
                                    println!("VPN can't stop - it's not running");
                                }
                                VpnStopResult::Fail(err) => {
                                    println!("VPN failed to stop: {err}");
                                }
                            }
                        }
                        "restart" => {
                            vpn_command_tx.send(VpnCommand::Restart).await.unwrap();
                        }
                        "status" => {
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

enum VpnState {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
}

fn start_vpn_handler(mut vpn_command_rx: Receiver<VpnCommand>) -> std::thread::JoinHandle<()> {
    println!("Starting VPN handler");
    std::thread::spawn(move || {
        let vpn_rt = tokio::runtime::Runtime::new().unwrap();
        vpn_rt.block_on(async {
            // Listen to the command channel
            println!("VPN: Listening for commands");
            let mut vpn_ctrl_sender: Option<UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>> = None;
            let mut vpn_state = Arc::new(std::sync::Mutex::new(VpnState::NotConnected));
            while let Some(command) = vpn_command_rx.recv().await {
                println!("VPN: Received command: {:?}", command);
                match command {
                    VpnCommand::Start(tx) => {
                        // Start the VPN

                        let entry_point = nym_vpn_lib::gateway_client::EntryPoint::Location {
                            location: "FR".to_string(),
                        };
                        let exit_point = nym_vpn_lib::gateway_client::ExitPoint::Location {
                            location: "FR".to_string(),
                        };
                        let mut nym_vpn = nym_vpn_lib::NymVpn::new(entry_point, exit_point);

                        let config = nym_vpn_lib::nym_config::OptionalSet::with_optional_env(
                            nym_vpn_lib::gateway_client::Config::default(),
                            nym_vpn_lib::gateway_client::Config::with_custom_api_url,
                            None,
                            "NYM_API",
                        );
                        nym_vpn.gateway_config = config;

                        let handle = nym_vpn_lib::spawn_nym_vpn(nym_vpn).unwrap();
                        {
                            *vpn_state.lock().unwrap() = VpnState::Connecting;
                        }

                        let nym_vpn_lib::NymVpnHandle {
                            vpn_ctrl_tx,
                            mut vpn_status_rx,
                            vpn_exit_rx,
                        } = handle;

                        vpn_ctrl_sender = Some(vpn_ctrl_tx);

                        tokio::spawn(async move {
                            while let Some(msg) = vpn_status_rx.next().await {
                                println!("Received status: {msg}");
                                match msg.downcast_ref::<nym_vpn_lib::TaskStatus>().unwrap() {
                                    nym_vpn_lib::TaskStatus::Ready => {
                                        vpn_state = VpnState::Connected;
                                    }
                                }
                            }
                        });

                        tokio::spawn(async move {
                            match vpn_exit_rx.await {
                                Ok(exit_res) => match exit_res {
                                    nym_vpn_lib::NymVpnExitStatusMessage::Stopped => {
                                        println!("VPN reports stopped");
                                    }
                                    nym_vpn_lib::NymVpnExitStatusMessage::Failed(err) => {
                                        println!("VPN reports exit fail: {err}");
                                    }
                                },
                                Err(err) => {
                                    println!("exit listener fail: {err}");
                                }
                            }
                        });

                        tx.send(VpnStartResult::Success).unwrap();
                    }
                    VpnCommand::Stop(tx) => {
                        // Stop the VPN
                        if let Some(ref mut vpn_ctrl_sender) = vpn_ctrl_sender {
                            let _ = vpn_ctrl_sender
                                .send(nym_vpn_lib::NymVpnCtrlMessage::Stop)
                                .await;
                            tx.send(VpnStopResult::Success).unwrap();
                        } else {
                            tx.send(VpnStopResult::NotRunning).unwrap();
                        }
                    }
                    VpnCommand::Restart => {
                        // Restart the VPN
                    }
                    VpnCommand::Status(_) => {
                        // Status
                    }
                }
            }
        });
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The idea here for explicly starting two separate runtimes is to make sure they are properly
    // separated. Looking ahead a little ideally it would be nice to be able for the command
    // interface to be able to forcefully terminate the vpn if needed.

    println!("main: starting command handler");
    let (command_handle, vpn_command_rx) = start_command_interface();

    println!("main: starting VPN handler");
    let vpn_handle = start_vpn_handler(vpn_command_rx);

    command_handle.join().unwrap();
    vpn_handle.join().unwrap();

    Ok(())
}
