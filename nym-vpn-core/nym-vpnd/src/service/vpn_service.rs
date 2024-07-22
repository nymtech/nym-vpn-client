// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::sync::Arc;

use futures::channel::{mpsc::UnboundedSender, oneshot::Receiver as OneshotReceiver};
use futures::SinkExt;
use nym_vpn_lib::credentials::import_credential;
use nym_vpn_lib::gateway_directory::{self, EntryPoint, ExitPoint};
use nym_vpn_lib::nym_bin_common::bin_info;
use nym_vpn_lib::{NodeIdentity, Recipient};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{broadcast, oneshot};
use tracing::{error, info};

use super::config::{
    self, create_config_file, create_data_dir, create_device_keys, read_config_file,
    write_config_file, ConfigSetupError, NymVpnServiceConfig, DEFAULT_CONFIG_FILE,
};
use super::error::{ConnectionFailedError, ImportCredentialError};
use super::exit_listener::VpnServiceExitListener;
use super::status_listener::VpnServiceStatusListener;

// The current state of the VPN service
#[derive(Debug, Clone)]
pub enum VpnState {
    NotConnected,
    Connecting,
    Connected(Box<VpnConnectedStateDetails>),
    Disconnecting,
    ConnectionFailed(ConnectionFailedError),
}

impl fmt::Display for VpnState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnState::NotConnected => write!(f, "NotConnected"),
            VpnState::Connecting => write!(f, "Connecting"),
            VpnState::Connected(details) => write!(f, "Connected({})", details),
            VpnState::Disconnecting => write!(f, "Disconnecting"),
            VpnState::ConnectionFailed(reason) => write!(f, "ConnectionFailed({})", reason),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VpnConnectedStateDetails {
    pub nym_address: Recipient,
    pub entry_gateway: NodeIdentity,
    pub exit_gateway: NodeIdentity,
    pub exit_ipr: Recipient,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
    pub since: time::OffsetDateTime,
}

impl fmt::Display for VpnConnectedStateDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "nym_address: {}, entry_gateway: {}, exit_gateway: {}, exit_ipr: {}, ipv4: {}, ipv6: {}, since: {}",
            self.nym_address,
            self.entry_gateway,
            self.exit_gateway,
            self.exit_ipr,
            self.ipv4,
            self.ipv6,
            self.since
        )
    }
}

#[allow(clippy::large_enum_variant)]
pub enum VpnServiceCommand {
    Connect(oneshot::Sender<VpnServiceConnectResult>, ConnectArgs),
    Disconnect(oneshot::Sender<VpnServiceDisconnectResult>),
    Status(oneshot::Sender<VpnServiceStatusResult>),
    Info(oneshot::Sender<VpnServiceInfoResult>),
    ImportCredential(
        oneshot::Sender<Result<Option<OffsetDateTime>, ImportCredentialError>>,
        Vec<u8>,
    ),
}

impl fmt::Display for VpnServiceCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnServiceCommand::Connect(_, args) => write!(f, "Connect {{ {args:?} }}"),
            VpnServiceCommand::Disconnect(_) => write!(f, "Disconnect"),
            VpnServiceCommand::Status(_) => write!(f, "Status"),
            VpnServiceCommand::Info(_) => write!(f, "Info"),
            VpnServiceCommand::ImportCredential(_, _) => write!(f, "ImportCredential"),
        }
    }
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
    #[allow(unused)]
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

// Respond with the current state of the VPN service. This is currently almost the same as VpnState,
// but it's conceptually not the same thing, so we keep them separate.
#[derive(Clone, Debug)]
pub enum VpnServiceStatusResult {
    NotConnected,
    Connecting,
    Connected(Box<ConnectedResultDetails>),
    Disconnecting,
    ConnectionFailed(ConnectionFailedError),
}

#[derive(Clone, Debug)]
pub struct VpnServiceInfoResult {
    pub version: String,
    pub build_timestamp: Option<time::OffsetDateTime>,
    pub triple: String,
    pub git_commit: String,
}

impl fmt::Display for VpnServiceStatusResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnServiceStatusResult::NotConnected => write!(f, "NotConnected"),
            VpnServiceStatusResult::Connecting => write!(f, "Connecting"),
            VpnServiceStatusResult::Connected(details) => write!(f, "Connected({})", details),
            VpnServiceStatusResult::Disconnecting => write!(f, "Disconnecting"),
            VpnServiceStatusResult::ConnectionFailed(reason) => {
                write!(f, "ConnectionFailed({})", reason)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectedResultDetails {
    pub nym_address: Recipient,
    pub entry_gateway: NodeIdentity,
    pub exit_gateway: NodeIdentity,
    pub exit_ipr: Recipient,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
    pub since: time::OffsetDateTime,
}

impl fmt::Display for ConnectedResultDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "nym_address: {}, entry_gateway: {}, exit_gateway: {}, exit_ipr: {}, ipv4: {}, ipv6: {}, since: {}",
            self.nym_address,
            self.entry_gateway,
            self.exit_gateway,
            self.exit_ipr,
            self.ipv4,
            self.ipv6,
            self.since
        )
    }
}

impl From<VpnConnectedStateDetails> for ConnectedResultDetails {
    fn from(details: VpnConnectedStateDetails) -> Self {
        ConnectedResultDetails {
            nym_address: details.nym_address,
            entry_gateway: details.entry_gateway,
            exit_gateway: details.exit_gateway,
            exit_ipr: details.exit_ipr,
            ipv4: details.ipv4,
            ipv6: details.ipv6,
            since: details.since,
        }
    }
}

impl From<Box<VpnConnectedStateDetails>> for Box<ConnectedResultDetails> {
    fn from(details: Box<VpnConnectedStateDetails>) -> Self {
        Box::new((*details).into())
    }
}

impl VpnServiceStatusResult {
    pub fn error(&self) -> Option<ConnectionFailedError> {
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
            VpnState::Connected(details) => VpnServiceStatusResult::Connected(details.into()),
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
    ConnectionFailed(ConnectionFailedError),
}

impl VpnServiceStateChange {
    pub fn error(&self) -> Option<ConnectionFailedError> {
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
            VpnState::Connected { .. } => VpnServiceStateChange::Connected,
            VpnState::Disconnecting => VpnServiceStateChange::Disconnecting,
            VpnState::ConnectionFailed(reason) => VpnServiceStateChange::ConnectionFailed(reason),
        }
    }
}

#[derive(Clone)]
pub(super) struct SharedVpnState {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
    vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
}

impl SharedVpnState {
    fn new(vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>) -> Self {
        Self {
            shared_vpn_state: Arc::new(std::sync::Mutex::new(VpnState::NotConnected)),
            vpn_state_changes_tx,
        }
    }

    pub(super) fn set(&self, state: VpnState) {
        info!("VPN: Setting shared state to {}", state);
        *self.shared_vpn_state.lock().unwrap() = state.clone();
        self.vpn_state_changes_tx.send(state.into()).ok();
    }

    fn get(&self) -> VpnState {
        self.shared_vpn_state.lock().unwrap().clone()
    }
}

pub(crate) struct NymVpnService {
    shared_vpn_state: SharedVpnState,

    // Listen for commands from the command interface, like the grpc listener that listens user
    // commands.
    vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,

    // Send commands to the actual vpn service task
    vpn_ctrl_sender: Option<UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>>,

    config_file: PathBuf,

    data_dir: PathBuf,
}

impl NymVpnService {
    pub(crate) fn new(
        vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
        vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,
    ) -> Self {
        let config_dir = std::env::var("NYM_VPND_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| config::default_config_dir());
        let config_file = config_dir.join(DEFAULT_CONFIG_FILE);
        let data_dir = std::env::var("NYM_VPND_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| config::default_data_dir());
        Self {
            shared_vpn_state: SharedVpnState::new(vpn_state_changes_tx),
            vpn_command_rx,
            vpn_ctrl_sender: None,
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
        self.shared_vpn_state.set(VpnState::Connecting);

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
                self.shared_vpn_state.set(VpnState::NotConnected);
                return VpnServiceConnectResult::Fail(err.to_string());
            }
        };

        info!("Using config: {}", config);

        // Make sure the data dir exists
        match create_data_dir(&self.data_dir) {
            Ok(()) => {}
            Err(err) => {
                self.shared_vpn_state.set(VpnState::NotConnected);
                return VpnServiceConnectResult::Fail(format!(
                    "Failed to create data directory {:?}: {}",
                    self.data_dir, err
                ));
            }
        }

        // Device specific keys
        match create_device_keys(&self.data_dir).await {
            Ok(()) => {}
            Err(err) => {
                self.shared_vpn_state.set(VpnState::NotConnected);
                return VpnServiceConnectResult::Fail(format!(
                    "Failed to create device keys in {:?}: {}",
                    self.data_dir, err
                ));
            }
        }

        let mut nym_vpn =
            nym_vpn_lib::NymVpn::new_mixnet_vpn(config.entry_point, config.exit_point);
        nym_vpn.gateway_config = gateway_directory::Config::new_from_env();
        nym_vpn.mixnet_client_config.mixnet_data_path = Some(self.data_dir.clone());
        nym_vpn.dns = options.dns;
        nym_vpn.disable_routing = options.disable_routing;
        nym_vpn.enable_two_hop = options.enable_two_hop;
        // TODO: add user agent to options struct so we can pass it from the connected client if we
        // want to
        nym_vpn.user_agent = Some(bin_info!().into());
        nym_vpn.mixnet_client_config.enable_poisson_rate = options.enable_poisson_rate;
        nym_vpn
            .mixnet_client_config
            .disable_background_cover_traffic = options.disable_background_cover_traffic;
        nym_vpn.mixnet_client_config.enable_credentials_mode = options.enable_credentials_mode;

        let handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(nym_vpn.into()).unwrap();

        let nym_vpn_lib::NymVpnHandle {
            vpn_ctrl_tx,
            vpn_status_rx,
            vpn_exit_rx,
        } = handle;

        self.vpn_ctrl_sender = Some(vpn_ctrl_tx);

        let (listener_vpn_status_tx, listener_vpn_status_rx) = futures::channel::mpsc::channel(16);
        let (listener_vpn_exit_tx, listener_vpn_exit_rx) = futures::channel::oneshot::channel();

        VpnServiceStatusListener::new(self.shared_vpn_state.clone())
            .start(vpn_status_rx, listener_vpn_status_tx)
            .await;

        VpnServiceExitListener::new(self.shared_vpn_state.clone())
            .start(vpn_exit_rx, listener_vpn_exit_tx)
            .await;

        let connect_handle = VpnServiceConnectHandle {
            listener_vpn_status_rx,
            listener_vpn_exit_rx,
        };

        VpnServiceConnectResult::Success(connect_handle)
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
            self.shared_vpn_state.set(VpnState::Disconnecting);
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
        self.shared_vpn_state.get().into()
    }

    async fn handle_info(&self) -> VpnServiceInfoResult {
        let bin_info = nym_vpn_lib::nym_bin_common::bin_info_local_vergen!();
        VpnServiceInfoResult {
            version: bin_info.build_version.to_string(),
            build_timestamp: time::OffsetDateTime::parse(bin_info.build_timestamp, &Rfc3339).ok(),
            triple: bin_info.cargo_triple.to_string(),
            git_commit: bin_info.commit_sha.to_string(),
        }
    }

    async fn handle_import_credential(
        &mut self,
        credential: Vec<u8>,
    ) -> Result<Option<OffsetDateTime>, ImportCredentialError> {
        if self.is_running() {
            return Err(ImportCredentialError::VpnRunning);
        }

        import_credential(credential, self.data_dir.clone())
            .await
            .map_err(|err| err.into())
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<()> {
        while let Some(command) = self.vpn_command_rx.recv().await {
            info!("VPN: Received command: {command}");
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
                VpnServiceCommand::Info(tx) => {
                    let result = self.handle_info().await;
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
