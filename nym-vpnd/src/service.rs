use std::sync::Arc;

use futures::SinkExt;
use futures::{channel::mpsc::UnboundedSender, StreamExt};
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub enum VpnState {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
}

#[derive(Debug)]
pub enum VpnServiceCommand {
    Connect(oneshot::Sender<VpnServiceConnectResult>),
    Disconnect(oneshot::Sender<VpnServiceDisconnectResult>),
    Status(oneshot::Sender<VpnServiceStatusResult>),
}

#[derive(Debug)]
pub enum VpnServiceConnectResult {
    Success,
    #[allow(unused)]
    Fail(String),
}

#[derive(Debug)]
pub enum VpnServiceDisconnectResult {
    Success,
    NotRunning,
    #[allow(unused)]
    Fail(String),
}

#[derive(Copy, Clone, Debug)]
pub enum VpnServiceStatusResult {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
}

pub fn start_vpn_service(
    vpn_command_rx: Receiver<VpnServiceCommand>,
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

#[cfg(unix)]
type VpnCtrlMessage = nym_vpn_lib::NymVpnCtrlMessage;
#[cfg(not(unix))]
type VpnCtrlMessage = crate::windows::VpnCtrlMessage;

struct NymVpnService {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
    vpn_command_rx: Receiver<VpnServiceCommand>,
    vpn_ctrl_sender: Option<UnboundedSender<VpnCtrlMessage>>,
}

impl NymVpnService {
    fn new(vpn_command_rx: Receiver<VpnServiceCommand>) -> Self {
        Self {
            shared_vpn_state: Arc::new(std::sync::Mutex::new(VpnState::NotConnected)),
            vpn_command_rx,
            vpn_ctrl_sender: None,
        }
    }

    #[cfg(not(unix))]
    async fn handle_connect(&mut self) -> VpnServiceConnectResult {
        println!("handle_connect");
    }

    #[cfg(unix)]
    async fn handle_connect(&mut self) -> VpnServiceConnectResult {
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

        VpnServiceStatusListener::new(self.shared_vpn_state.clone())
            .start(vpn_status_rx)
            .await;

        VpnServiceExitListener::new(self.shared_vpn_state.clone())
            .start(vpn_exit_rx)
            .await;

        VpnServiceConnectResult::Success
    }

    fn set_shared_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }

    #[cfg(not(unix))]
    async fn handle_disconnect(&mut self) -> VpnServiceDisconnectResult {
        println!("handle_disconnect");
    }

    #[cfg(unix)]
    async fn handle_disconnect(&mut self) -> VpnServiceDisconnectResult {
        // To handle the mutable borrow we set the state separate from the sending the stop
        // message, including the logical check for the ctrl sender twice.
        let is_running = self.vpn_ctrl_sender.is_some();

        if is_running {
            self.set_shared_state(VpnState::Disconnecting);
        }

        if let Some(ref mut vpn_ctrl_sender) = self.vpn_ctrl_sender {
            let _ = vpn_ctrl_sender
                .send(nym_vpn_lib::NymVpnCtrlMessage::Stop)
                .await;
            VpnServiceDisconnectResult::Success
        } else {
            VpnServiceDisconnectResult::NotRunning
        }
    }

    async fn handle_status(&self) -> VpnServiceStatusResult {
        match *self.shared_vpn_state.lock().unwrap() {
            VpnState::NotConnected => VpnServiceStatusResult::NotConnected,
            VpnState::Connecting => VpnServiceStatusResult::Connecting,
            VpnState::Connected => VpnServiceStatusResult::Connected,
            VpnState::Disconnecting => VpnServiceStatusResult::Disconnecting,
        }
    }

    async fn run(mut self) {
        while let Some(command) = self.vpn_command_rx.recv().await {
            println!("VPN: Received command: {:?}", command);
            match command {
                VpnServiceCommand::Connect(tx) => {
                    let result = self.handle_connect().await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::Disconnect(tx) => {
                    let result = self.handle_disconnect().await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::Status(tx) => {
                    let result = self.handle_status().await;
                    tx.send(result).unwrap();
                }
            }
        }
    }
}

struct VpnServiceStatusListener {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
}

impl VpnServiceStatusListener {
    fn new(shared_vpn_state: Arc<std::sync::Mutex<VpnState>>) -> Self {
        Self { shared_vpn_state }
    }

    #[cfg(not(unix))]
    async fn start(
        self,
        mut vpn_status_rx: futures::channel::mpsc::Receiver<
            Box<dyn std::error::Error + Send + Sync>,
        >,
    ) {
        println!("VPN status listener not implemented for Windows");
    }

    #[cfg(unix)]
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
                    nym_vpn_lib::TaskStatus::Ready
                    | nym_vpn_lib::TaskStatus::ReadyWithGateway(_) => {
                        println!("VPN status: connected");
                        self.set_shared_state(VpnState::Connected);
                    }
                }
            }
        });
    }

    fn set_shared_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }
}

struct VpnServiceExitListener {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
}

impl VpnServiceExitListener {
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
                        self.set_shared_state(VpnState::NotConnected);
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

    fn set_shared_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }
}
