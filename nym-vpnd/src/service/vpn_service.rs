// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use nym_vpn_lib::credentials::import_credential;
use nym_vpn_lib::gateway_directory::{self};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot;
use tracing::info;

use super::config::{
    create_config_file, create_data_dir, read_config_file, ConfigSetupError, NymVpnServiceConfig,
    DEFAULT_CONFIG_DIR, DEFAULT_CONFIG_FILE, DEFAULT_DATA_DIR,
};
use super::exit_listener::VpnServiceExitListener;
use super::status_listener::VpnServiceStatusListener;

#[derive(Debug, Clone)]
pub enum VpnState {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
    #[allow(unused)]
    ConnectionFailed(String),
}

#[derive(Debug)]
pub enum VpnServiceCommand {
    Connect(oneshot::Sender<VpnServiceConnectResult>, ConnectArgs),
    Disconnect(oneshot::Sender<VpnServiceDisconnectResult>),
    Status(oneshot::Sender<VpnServiceStatusResult>),
    ImportCredential(
        oneshot::Sender<VpnServiceImportUserCredentialResult>,
        Vec<u8>,
    ),
}

#[derive(Debug)]
pub enum ConnectArgs {
    // Read the entry and exit points from the config file.
    Default,
    Custom(CustomConnectArgs),
}

#[derive(Debug)]
pub struct CustomConnectArgs {
    pub entry: String,
    // pub exit: String,
}

#[derive(Debug)]
pub enum VpnServiceConnectResult {
    Success,
    Fail(String),
}

impl VpnServiceConnectResult {
    pub fn is_success(&self) -> bool {
        matches!(self, VpnServiceConnectResult::Success)
    }
}

#[derive(Debug)]
pub enum VpnServiceDisconnectResult {
    Success,
    NotRunning,
    #[allow(unused)]
    Fail(String),
}

impl VpnServiceDisconnectResult {
    pub fn is_success(&self) -> bool {
        matches!(self, VpnServiceDisconnectResult::Success)
    }
}

#[derive(Clone, Debug)]
pub enum VpnServiceStatusResult {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
    ConnectionFailed(String),
}

#[derive(Debug)]
pub enum VpnServiceImportUserCredentialResult {
    Success,
    Fail(String),
}

impl VpnServiceImportUserCredentialResult {
    pub fn is_success(&self) -> bool {
        matches!(self, VpnServiceImportUserCredentialResult::Success)
    }
}

pub(super) struct NymVpnService {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
    vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,
    vpn_ctrl_sender: Option<UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>>,
    config_file: PathBuf,
    data_dir: PathBuf,
}

impl NymVpnService {
    pub(super) fn new(vpn_command_rx: UnboundedReceiver<VpnServiceCommand>) -> Self {
        let config_dir = std::env::var("NYM_VPND_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_CONFIG_DIR));
        let config_file = config_dir.join(DEFAULT_CONFIG_FILE);
        let data_dir = std::env::var("NYM_VPND_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_DATA_DIR));
        Self {
            shared_vpn_state: Arc::new(std::sync::Mutex::new(VpnState::NotConnected)),
            vpn_command_rx,
            vpn_ctrl_sender: None,
            config_file,
            data_dir,
        }
    }

    fn try_setup_config(&self) -> std::result::Result<NymVpnServiceConfig, ConfigSetupError> {
        // If the config file does not exit, create it
        let config = if self.config_file.exists() {
            read_config_file(&self.config_file)?
        } else {
            create_config_file(&self.config_file, NymVpnServiceConfig::default())?
        };
        Ok(config)
    }

    async fn handle_connect(&mut self, connect_args: ConnectArgs) -> VpnServiceConnectResult {
        self.set_shared_state(VpnState::Connecting);

        let entry = match connect_args {
            ConnectArgs::Default => None,
            ConnectArgs::Custom(CustomConnectArgs { entry }) => {
                info!("Using custom entry point: {}", entry);
                Some(entry)
            }
        };

        // TODO: pass in location to the config file
        let config = match self.try_setup_config() {
            Ok(config) => config,
            Err(err) => {
                self.set_shared_state(VpnState::NotConnected);
                return VpnServiceConnectResult::Fail(err.to_string());
            }
        };

        // Make sure the data dir exists
        match create_data_dir(&self.data_dir) {
            Ok(()) => {}
            Err(err) => {
                self.set_shared_state(VpnState::NotConnected);
                return VpnServiceConnectResult::Fail(format!(
                    "Failed to create data directory {:?}: {}",
                    self.data_dir, err
                ));
            }
        }

        let mut nym_vpn = nym_vpn_lib::NymVpn::new(config.entry_point, config.exit_point);
        nym_vpn.gateway_config = gateway_directory::Config::new_from_env();
        nym_vpn.mixnet_data_path = Some(self.data_dir.clone());

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
        info!("VPN: Setting shared state to {:?}", state);
        *self.shared_vpn_state.lock().unwrap() = state;
    }

    fn is_running(&self) -> bool {
        self.vpn_ctrl_sender
            .as_ref()
            .map(|s| !s.is_closed())
            .unwrap_or(false)
    }

    async fn handle_disconnect(&mut self) -> VpnServiceDisconnectResult {
        // To handle the mutable borrow we set the state separate from the sending the stop message
        if self.is_running() {
            self.set_shared_state(VpnState::Disconnecting);
        } else {
            return VpnServiceDisconnectResult::NotRunning;
        }

        if let Some(ref mut vpn_ctrl_sender) = self.vpn_ctrl_sender {
            vpn_ctrl_sender
                .send(nym_vpn_lib::NymVpnCtrlMessage::Stop)
                .await
                .ok();
            VpnServiceDisconnectResult::Success
        } else {
            VpnServiceDisconnectResult::NotRunning
        }
    }

    async fn handle_status(&self) -> VpnServiceStatusResult {
        match self.shared_vpn_state.lock().unwrap().clone() {
            VpnState::NotConnected => VpnServiceStatusResult::NotConnected,
            VpnState::Connecting => VpnServiceStatusResult::Connecting,
            VpnState::Connected => VpnServiceStatusResult::Connected,
            VpnState::Disconnecting => VpnServiceStatusResult::Disconnecting,
            VpnState::ConnectionFailed(reason) => VpnServiceStatusResult::ConnectionFailed(reason),
        }
    }

    async fn handle_import_credential(
        &mut self,
        credential: Vec<u8>,
    ) -> VpnServiceImportUserCredentialResult {
        if self.is_running() {
            return VpnServiceImportUserCredentialResult::Fail(
                "Can't import credential while VPN is running".to_string(),
            );
        }

        match import_credential(credential, self.data_dir.clone()).await {
            Ok(()) => VpnServiceImportUserCredentialResult::Success,
            Err(err) => VpnServiceImportUserCredentialResult::Fail(err.to_string()),
        }
    }

    pub(super) async fn run(mut self) -> anyhow::Result<()> {
        while let Some(command) = self.vpn_command_rx.recv().await {
            info!("VPN: Received command: {:?}", command);
            match command {
                VpnServiceCommand::Connect(tx, connect_args) => {
                    let result = self.handle_connect(connect_args).await;
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
                VpnServiceCommand::ImportCredential(tx, credential) => {
                    let result = self.handle_import_credential(credential).await;
                    tx.send(result).unwrap();
                }
            }
        }
        Ok(())
    }
}
