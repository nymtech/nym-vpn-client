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

struct NymVpnService {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
    vpn_command_rx: Receiver<command_interface::VpnCommand>,
    vpn_ctrl_sender: Option<UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>>,
}

impl NymVpnService {
    fn new(vpn_command_rx: Receiver<command_interface::VpnCommand>) -> Self {
        Self {
            shared_vpn_state: Arc::new(std::sync::Mutex::new(VpnState::NotConnected)),
            vpn_command_rx,
            vpn_ctrl_sender: None,
        }
    }

    async fn handle_connect(&mut self) -> command_interface::VpnConnectResult {
        // Start the VPN
        // TODO: all of this is hardcoded for now
        self.set_shared_state(VpnState::Connecting);

        let mut nym_vpn = nym_vpn_lib::NymVpn::new(
            nym_vpn_lib::gateway_client::EntryPoint::Location {
                location: "FR".to_string(),
            },
            nym_vpn_lib::gateway_client::ExitPoint::Location {
                location: "FR".to_string(),
            },
        );

        nym_vpn.gateway_config = nym_vpn_lib::nym_config::OptionalSet::with_optional_env(
            nym_vpn_lib::gateway_client::Config::default(),
            nym_vpn_lib::gateway_client::Config::with_custom_api_url,
            None,
            "NYM_API",
        );

        let handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(nym_vpn).unwrap();

        let nym_vpn_lib::NymVpnHandle {
            vpn_ctrl_tx,
            vpn_status_rx,
            vpn_exit_rx,
        } = handle;

        self.vpn_ctrl_sender = Some(vpn_ctrl_tx);

        VpnStatusListener::new(self.shared_vpn_state.clone())
            .start(vpn_status_rx)
            .await;

        VpnExitListener::new(self.shared_vpn_state.clone())
            .start(vpn_exit_rx)
            .await;

        command_interface::VpnConnectResult::Success
    }

    fn set_shared_state(&mut self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }

    async fn handle_disconnect(&mut self) -> command_interface::VpnDisconnectResult {
        if let Some(ref mut vpn_ctrl_sender) = self.vpn_ctrl_sender {
            {
                *self.shared_vpn_state.lock().unwrap() = VpnState::Disconnecting;
            }
            let _ = vpn_ctrl_sender
                .send(nym_vpn_lib::NymVpnCtrlMessage::Stop)
                .await;
            command_interface::VpnDisconnectResult::Success
        } else {
            command_interface::VpnDisconnectResult::NotRunning
        }
    }

    async fn handle_status(&self) -> command_interface::VpnStatusResult {
        // Current status of the vpn
        let state = { self.shared_vpn_state.lock().unwrap().clone() };
        match state {
            VpnState::NotConnected => command_interface::VpnStatusResult::NotConnected,
            VpnState::Connecting => command_interface::VpnStatusResult::Connecting,
            VpnState::Connected => command_interface::VpnStatusResult::Connected,
            VpnState::Disconnecting => command_interface::VpnStatusResult::Disconnecting,
        }
    }

    async fn run(mut self) {
        while let Some(command) = self.vpn_command_rx.recv().await {
            println!("VPN: Received command: {:?}", command);
            match command {
                command_interface::VpnCommand::Connect(tx) => {
                    let result = self.handle_connect().await;
                    tx.send(result).unwrap();
                }
                command_interface::VpnCommand::Disconnect(tx) => {
                    let result = self.handle_disconnect().await;
                    tx.send(result).unwrap();
                }
                command_interface::VpnCommand::Status(tx) => {
                    let result = self.handle_status().await;
                    tx.send(result).unwrap();
                }
            }
        }
    }
}

struct VpnStatusListener {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
}

impl VpnStatusListener {
    fn new(shared_vpn_state: Arc<std::sync::Mutex<VpnState>>) -> Self {
        Self { shared_vpn_state }
    }

    async fn start(
        self,
        mut vpn_status_rx: futures::channel::mpsc::Receiver<
            Box<dyn std::error::Error + Send + Sync>,
        >,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = vpn_status_rx.next().await {
                println!("Received status: {msg}");
                match msg.downcast_ref::<nym_vpn_lib::TaskStatus>().unwrap() {
                    nym_vpn_lib::TaskStatus::Ready => {
                        println!("VPN status: connected");
                        self.set_state(VpnState::Connected);
                    }
                }
            }
        });
    }

    fn set_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }
}

struct VpnExitListener {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
}

impl VpnExitListener {
    fn new(shared_vpn_state: Arc<std::sync::Mutex<VpnState>>) -> Self {
        Self { shared_vpn_state }
    }

    async fn start(
        self,
        vpn_exit_rx: futures::channel::oneshot::Receiver<nym_vpn_lib::NymVpnExitStatusMessage>,
    ) {
        tokio::spawn(async move {
            match vpn_exit_rx.await {
                Ok(exit_res) => match exit_res {
                    nym_vpn_lib::NymVpnExitStatusMessage::Stopped => {
                        println!("VPN exit: stopped");
                        self.set_state(VpnState::NotConnected);
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
    }

    fn set_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }
}

pub fn start_vpn_service(
    vpn_command_rx: Receiver<command_interface::VpnCommand>,
) -> std::thread::JoinHandle<()> {
    println!("Starting VPN handler");
    std::thread::spawn(move || {
        let vpn_rt = tokio::runtime::Runtime::new().unwrap();
        vpn_rt.block_on(async {
            // Listen to the command channel
            println!("VPN: Listening for commands");
            let vpn_service = NymVpnService::new(vpn_command_rx);
            vpn_service.run().await;
        });
    })
}
