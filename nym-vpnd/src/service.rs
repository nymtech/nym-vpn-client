use std::sync::Arc;

use futures::SinkExt;
use futures::{channel::mpsc::UnboundedSender, StreamExt};
use tokio::sync::mpsc::Receiver;

use crate::command_interface;

#[derive(Debug, Clone)]
pub enum VpnState {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
}

pub fn start_vpn_service(
    mut vpn_command_rx: Receiver<command_interface::VpnCommand>,
) -> std::thread::JoinHandle<()> {
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
                    command_interface::VpnCommand::Start(tx) => {
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

                        let handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(nym_vpn).unwrap();
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

                        tx.send(command_interface::VpnStartResult::Success).unwrap();
                    }
                    command_interface::VpnCommand::Stop(tx) => {
                        // Stop the VPN
                        if let Some(ref mut vpn_ctrl_sender) = vpn_ctrl_sender {
                            {
                                *vpn_state.lock().unwrap() = VpnState::Disconnecting;
                            }
                            let _ = vpn_ctrl_sender
                                .send(nym_vpn_lib::NymVpnCtrlMessage::Stop)
                                .await;
                            tx.send(command_interface::VpnStopResult::Success).unwrap();
                        } else {
                            tx.send(command_interface::VpnStopResult::NotRunning)
                                .unwrap();
                        }
                    }
                    command_interface::VpnCommand::Restart => {
                        // Restart the VPN
                    }
                    command_interface::VpnCommand::Status(tx) => {
                        // Current status of the vpn
                        let state = { vpn_state.lock().unwrap().clone() };
                        let vpn_status_result = match state {
                            VpnState::NotConnected => {
                                command_interface::VpnStatusResult::NotConnected
                            }
                            VpnState::Connecting => command_interface::VpnStatusResult::Connecting,
                            VpnState::Connected => command_interface::VpnStatusResult::Connected,
                            VpnState::Disconnecting => {
                                command_interface::VpnStatusResult::Disconnecting
                            }
                        };
                        tx.send(vpn_status_result).unwrap();
                    }
                }
            }
        });
    })
}
