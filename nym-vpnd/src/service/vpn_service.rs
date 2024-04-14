// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use nym_vpn_lib::gateway_directory;
use nym_vpn_lib::nym_config::OptionalSet;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot;
use tracing::info;

use super::exit_listener::VpnServiceExitListener;
use super::status_listener::VpnServiceStatusListener;

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

#[derive(Copy, Clone, Debug)]
pub enum VpnServiceStatusResult {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
}

// Config file saves as toml file
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct NymVpnServiceConfig {
    entry_point: gateway_directory::EntryPoint,
    exit_point: gateway_directory::ExitPoint,
}

impl Default for NymVpnServiceConfig {
    fn default() -> Self {
        Self {
            entry_point: gateway_directory::EntryPoint::Random,
            exit_point: gateway_directory::ExitPoint::Random,
        }
    }
}

pub(super) struct NymVpnService {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
    vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,
    vpn_ctrl_sender: Option<UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>>,
    config_file: PathBuf,
    #[allow(unused)]
    data_dir: PathBuf,
}

const DEFAULT_DATA_DIR: &str = "/var/lib/nym-vpnd";
const DEFAULT_CONFIG_FILE: &str = "/etc/nym/nym-vpnd.toml";

impl NymVpnService {
    pub(super) fn new(vpn_command_rx: UnboundedReceiver<VpnServiceCommand>) -> Self {
        let config_file = std::env::var("NYM_VPND_CONFIG_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_CONFIG_FILE));
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

    fn try_setup_config(
        &self,
    ) -> std::result::Result<NymVpnServiceConfig, VpnServiceConnectResult> {
        // If the config file does not exit, create it
        let config = if self.config_file.exists() {
            let config: NymVpnServiceConfig = match std::fs::read_to_string(&self.config_file) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(err) => {
                        return Err(VpnServiceConnectResult::Fail(format!(
                            "Failed to parse config file {:?}: {:?}",
                            self.config_file, err
                        )));
                    }
                },
                Err(err) => {
                    return Err(VpnServiceConnectResult::Fail(format!(
                        "Failed to read config file {:?}: {:?}",
                        self.config_file, err
                    )));
                }
            };
            config
        } else {
            let config = NymVpnServiceConfig::default();
            let config_str = toml::to_string(&config).unwrap();
            // Create path
            match self.config_file.parent() {
                Some(parent) => {
                    if let Err(err) = std::fs::create_dir_all(parent) {
                        return Err(VpnServiceConnectResult::Fail(format!(
                            "Failed to create parent directory {:?}: {:?}",
                            parent, err
                        )));
                    }
                }
                None => {
                    return Err(VpnServiceConnectResult::Fail(format!(
                        "Failed to get parent directory of {:?}",
                        self.config_file
                    )));
                }
            }
            match std::fs::write(&self.config_file, config_str) {
                Ok(_) => {
                    info!("Config file created at {:?}", self.config_file);
                }
                Err(err) => {
                    return Err(VpnServiceConnectResult::Fail(format!(
                        "Failed to create config file {:?}: {:?}",
                        self.config_file, err
                    )));
                }
            }
            config
        };
        Ok(config)
    }

    async fn handle_connect(&mut self) -> VpnServiceConnectResult {
        self.set_shared_state(VpnState::Connecting);

        let config = match self.try_setup_config() {
            Ok(config) => config,
            Err(err) => {
                self.set_shared_state(VpnState::NotConnected);
                return err;
            }
        };

        let mut nym_vpn = nym_vpn_lib::NymVpn::new(config.entry_point, config.exit_point);

        nym_vpn.gateway_config = gateway_directory::Config::default()
            .with_optional_env(
                gateway_directory::Config::with_custom_api_url,
                None,
                "NYM_API",
            )
            .with_optional_env(
                gateway_directory::Config::with_custom_explorer_url,
                None,
                "EXPLORER_API",
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

    pub(super) async fn run(mut self) -> anyhow::Result<()> {
        while let Some(command) = self.vpn_command_rx.recv().await {
            info!("VPN: Received command: {:?}", command);
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
        Ok(())
    }
}
