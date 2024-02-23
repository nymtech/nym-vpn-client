use std::path::Path;
use std::sync::Arc;

use futures::SinkExt;
use futures::{channel::mpsc::UnboundedSender, StreamExt};
use tokio::io::AsyncWriteExt;
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

#[derive(Copy, Clone, Debug)]
enum VpnStatusResult {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
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
                        "connect" => {
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
                        "disconnect" => {
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
                        "status" => {
                            let (tx, rx) = oneshot::channel();
                            vpn_command_tx.send(VpnCommand::Status(tx)).await.unwrap();
                            println!("Sent status command to VPN");
                            println!("Waiting for response");
                            let status = rx.await.unwrap();
                            println!("VPN status: {:?}", status);
                            socket.write_all(format!("{:?}", status).as_bytes()).await.unwrap();
                        }
                        command => println!("Unknown command: {}", command),
                    }
                }
                Err(e) => println!("Failed to read from socket; err = {:?}", e),
            }
        });
    }
}

#[derive(Debug, Clone)]
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
            let vpn_state = Arc::new(std::sync::Mutex::new(VpnState::NotConnected));
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

                        let vpn_state_1 = vpn_state.clone();
                        tokio::spawn(async move {
                            while let Some(msg) = vpn_status_rx.next().await {
                                println!("Received status: {msg}");
                                match msg.downcast_ref::<nym_vpn_lib::TaskStatus>().unwrap() {
                                    nym_vpn_lib::TaskStatus::Ready => {
                                        println!("VPN status: connected");
                                        *vpn_state_1.lock().unwrap() = VpnState::Connected;
                                    }
                                }
                            }
                        });

                        let vpn_state_2 = vpn_state.clone();
                        tokio::spawn(async move {
                            match vpn_exit_rx.await {
                                Ok(exit_res) => match exit_res {
                                    nym_vpn_lib::NymVpnExitStatusMessage::Stopped => {
                                        println!("VPN exit: stopped");
                                        {
                                            *vpn_state_2.lock().unwrap() = VpnState::NotConnected;
                                        }
                                    }
                                    nym_vpn_lib::NymVpnExitStatusMessage::Failed(err) => {
                                        println!("VPN exit: fail: {err}");
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
                            {
                                *vpn_state.lock().unwrap() = VpnState::Disconnecting;
                            }
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
                    VpnCommand::Status(tx) => {
                        // Current status of the vpn
                        let state = { vpn_state.lock().unwrap().clone() };
                        let vpn_status_result = match state {
                            VpnState::NotConnected => VpnStatusResult::NotConnected,
                            VpnState::Connecting => VpnStatusResult::Connecting,
                            VpnState::Connected => VpnStatusResult::Connected,
                            VpnState::Disconnecting => VpnStatusResult::Disconnecting,
                        };
                        tx.send(vpn_status_result).unwrap();
                    }
                }
            }
        });
    })
}

pub fn setup_logging() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    nym_vpn_lib::nym_config::defaults::setup_env(Some("/home/jon/src/nym/nym/envs/sandbox.env"));

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
