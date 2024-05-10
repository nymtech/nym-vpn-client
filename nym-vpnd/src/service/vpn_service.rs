// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use futures::channel::{mpsc::UnboundedSender, oneshot::Receiver as OneshotReceiver};
use futures::SinkExt;
use nym_vpn_lib::credentials::import_credential;
use nym_vpn_lib::gateway_directory::{self, EntryPoint, ExitPoint};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{broadcast, oneshot};
use tracing::{error, info};

use super::config::{
    create_config_file, create_data_dir, read_config_file, write_config_file, ConfigSetupError,
    NymVpnServiceConfig, DEFAULT_CONFIG_DIR, DEFAULT_CONFIG_FILE, DEFAULT_DATA_DIR,
};
use super::exit_listener::VpnServiceExitListener;
use super::status_listener::VpnServiceStatusListener;

// The current state of the VPN service
#[derive(Debug, Clone)]
pub enum VpnState {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
    ConnectionFailed(String),
}

#[allow(clippy::large_enum_variant)]
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
pub struct ConnectArgs {
    pub entry: Option<gateway_directory::EntryPoint>,
    pub exit: Option<gateway_directory::ExitPoint>,
    pub options: ConnectOptions,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConnectOptions {
    pub(crate) dns: Option<IpAddr>,
    pub(crate) disable_routing: bool,
    pub(crate) enable_two_hop: bool,
    pub(crate) enable_poisson_rate: bool,
    pub(crate) disable_background_cover_traffic: bool,
    pub(crate) enable_credentials_mode: bool,
}

#[derive(Debug)]
pub enum VpnServiceConnectResult {
    Success(VpnServiceConnectHandle),
    Fail(String),
}

impl VpnServiceConnectResult {
    pub fn is_success(&self) -> bool {
        matches!(self, VpnServiceConnectResult::Success(_))
    }
}

#[derive(Debug)]
pub struct VpnServiceConnectHandle {
    pub listener_vpn_status_rx: nym_vpn_lib::StatusReceiver,
    pub listener_vpn_exit_rx: OneshotReceiver<nym_vpn_lib::NymVpnExitStatusMessage>,
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

// Respond with the current state of the VPN service. This is currently almos the same as VpnState,
// but it's conceptually not the same thing, so we keep them separate.
#[derive(Clone, Debug)]
pub enum VpnServiceStatusResult {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
    ConnectionFailed(String),
}

impl VpnServiceStatusResult {
    pub fn error(&self) -> Option<String> {
        match self {
            VpnServiceStatusResult::ConnectionFailed(reason) => Some(reason.clone()),
            _ => None,
        }
    }
}

impl From<VpnState> for VpnServiceStatusResult {
    fn from(state: VpnState) -> Self {
        match state {
            VpnState::NotConnected => VpnServiceStatusResult::NotConnected,
            VpnState::Connecting => VpnServiceStatusResult::Connecting,
            VpnState::Connected => VpnServiceStatusResult::Connected,
            VpnState::Disconnecting => VpnServiceStatusResult::Disconnecting,
            VpnState::ConnectionFailed(reason) => VpnServiceStatusResult::ConnectionFailed(reason),
        }
    }
}

#[derive(Clone, Debug)]
pub enum VpnServiceStateChange {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
    ConnectionFailed(String),
}

impl VpnServiceStateChange {
    pub fn error(&self) -> Option<String> {
        match self {
            VpnServiceStateChange::ConnectionFailed(reason) => Some(reason.clone()),
            _ => None,
        }
    }
}

impl From<VpnState> for VpnServiceStateChange {
    fn from(state: VpnState) -> Self {
        match state {
            VpnState::NotConnected => VpnServiceStateChange::NotConnected,
            VpnState::Connecting => VpnServiceStateChange::Connecting,
            VpnState::Connected => VpnServiceStateChange::Connected,
            VpnState::Disconnecting => VpnServiceStateChange::Disconnecting,
            VpnState::ConnectionFailed(reason) => VpnServiceStateChange::ConnectionFailed(reason),
        }
    }
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

    // Listen for commands from the command interface, like the grpc listener that listens user
    // commands.
    vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,

    // Send commands to the actual vpn service task
    vpn_ctrl_sender: Option<UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>>,

    // Broadcast connection state changes to whoever is interested, which typically is the command
    // interface
    vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,

    config_file: PathBuf,

    data_dir: PathBuf,
}

impl NymVpnService {
    pub(super) fn new(
        vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
        vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,
    ) -> Self {
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
            vpn_state_changes_tx,
            config_file,
            data_dir,
        }
    }

    fn try_setup_config(
        &self,
        entry: Option<gateway_directory::EntryPoint>,
        exit: Option<gateway_directory::ExitPoint>,
    ) -> std::result::Result<NymVpnServiceConfig, ConfigSetupError> {
        // If the config file does not exit, create it
        let config = if self.config_file.exists() {
            let mut read_config = read_config_file(&self.config_file)
                .map_err(|err| {
                    error!(
                        "Failed to read config file, resetting to defaults: {:?}",
                        err
                    );
                })
                .unwrap_or_default();
            read_config.entry_point = entry.unwrap_or(read_config.entry_point);
            read_config.exit_point = exit.unwrap_or(read_config.exit_point);
            write_config_file(&self.config_file, &read_config)?;
            read_config
        } else {
            let config = NymVpnServiceConfig {
                entry_point: entry.unwrap_or(EntryPoint::Random),
                exit_point: exit.unwrap_or(ExitPoint::Random),
            };
            create_config_file(&self.config_file, config)?
        };
        Ok(config)
    }

    async fn handle_connect(&mut self, connect_args: ConnectArgs) -> VpnServiceConnectResult {
        self.set_shared_state(VpnState::Connecting);

        let ConnectArgs {
            entry,
            exit,
            options,
        } = connect_args;
        info!("Using entry point: {:?}", entry);
        info!("Using exit point: {:?}", exit);
        info!("Using options: {:?}", options);

        let config = match self.try_setup_config(entry, exit) {
            Ok(config) => config,
            Err(err) => {
                self.set_shared_state(VpnState::NotConnected);
                return VpnServiceConnectResult::Fail(err.to_string());
            }
        };

        info!("Using config: {:?}", config);

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

        let mut nym_vpn =
            nym_vpn_lib::NymVpn::new_mixnet_vpn(config.entry_point, config.exit_point);
        nym_vpn.gateway_config = gateway_directory::Config::new_from_env();
        nym_vpn.vpn_config.mixnet_data_path = Some(self.data_dir.clone());
        nym_vpn.dns = options.dns;
        nym_vpn.disable_routing = options.disable_routing;
        nym_vpn.enable_two_hop = options.enable_two_hop;
        nym_vpn.vpn_config.enable_poisson_rate = options.enable_poisson_rate;
        nym_vpn.vpn_config.disable_background_cover_traffic =
            options.disable_background_cover_traffic;
        nym_vpn.vpn_config.enable_credentials_mode = options.enable_credentials_mode;

        let handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(nym_vpn.into()).unwrap();

        let nym_vpn_lib::NymVpnHandle {
            vpn_ctrl_tx,
            vpn_status_rx,
            vpn_exit_rx,
        } = handle;

        self.vpn_ctrl_sender = Some(vpn_ctrl_tx);

        let (listener_vpn_status_tx, listener_vpn_status_rx) = futures::channel::mpsc::channel(16);
        let (listener_vpn_exit_tx, listener_vpn_exit_rx) = futures::channel::oneshot::channel();

        VpnServiceStatusListener::new(
            self.vpn_state_changes_tx.clone(),
            self.shared_vpn_state.clone(),
        )
        .start(vpn_status_rx, listener_vpn_status_tx)
        .await;

        VpnServiceExitListener::new(
            self.vpn_state_changes_tx.clone(),
            self.shared_vpn_state.clone(),
        )
        .start(vpn_exit_rx, listener_vpn_exit_tx)
        .await;

        let connect_handle = VpnServiceConnectHandle {
            listener_vpn_status_rx,
            listener_vpn_exit_rx,
        };

        VpnServiceConnectResult::Success(connect_handle)
    }

    fn set_shared_state(&self, state: VpnState) {
        info!("VPN: Setting shared state to {:?}", state);
        *self.shared_vpn_state.lock().unwrap() = state.clone();
        self.vpn_state_changes_tx.send(state.into()).ok();
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
        self.shared_vpn_state.lock().unwrap().clone().into()
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
