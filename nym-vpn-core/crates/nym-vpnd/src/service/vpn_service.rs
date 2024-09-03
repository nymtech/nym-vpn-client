// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::Arc,
};

use bip39::Mnemonic;
use futures::{
    channel::{mpsc::UnboundedSender, oneshot::Receiver as OneshotReceiver},
    SinkExt,
};
use nym_vpn_api_client::{
    response::{NymVpnAccountSummaryResponse, NymVpnDevice},
    types::VpnApiAccount,
};
use nym_vpn_lib::{
    credentials::import_credential,
    gateway_directory::{self, EntryPoint, ExitPoint},
    nym_config::defaults::NymNetworkDetails,
    GenericNymVpnConfig, MixnetClientConfig, NodeIdentity, Recipient,
};
use nym_vpn_store::keys::KeyStore as _;
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::sync::{broadcast, mpsc::UnboundedReceiver, oneshot};
use tracing::{debug, error, info};
use url::Url;

use super::{
    config::{
        self, create_config_file, create_data_dir, read_config_file, write_config_file,
        ConfigSetupError, NymVpnServiceConfig, DEFAULT_CONFIG_FILE,
    },
    error::{AccountError, ConnectionFailedError, ImportCredentialError},
    exit_listener::VpnServiceExitListener,
    status_listener::VpnServiceStatusListener,
};

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
pub struct MixConnectedStateDetails {
    pub nym_address: Recipient,
    pub exit_ipr: Recipient,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
}

#[derive(Debug, Clone)]
pub struct WgConnectedStateDetails {
    pub entry_ipv4: Ipv4Addr,
    pub exit_ipv4: Ipv4Addr,
}

#[derive(Debug, Clone)]
pub enum ConnectedStateDetails {
    Mix(Box<MixConnectedStateDetails>),
    Wg(WgConnectedStateDetails),
}

impl fmt::Display for ConnectedStateDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mix(details) => {
                write!(
                    f,
                    "nym_address: {}, exit_ipr: {}, ipv4: {}, ipv6: {}",
                    details.nym_address, details.exit_ipr, details.ipv4, details.ipv6
                )
            }
            Self::Wg(details) => {
                write!(
                    f,
                    "entry_ipv4: {}, exit_ipv4: {}",
                    details.entry_ipv4, details.exit_ipv4
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct VpnConnectedStateDetails {
    pub entry_gateway: NodeIdentity,
    pub exit_gateway: NodeIdentity,
    pub specific_details: ConnectedStateDetails,
    pub since: time::OffsetDateTime,
}

impl fmt::Display for VpnConnectedStateDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "entry_gateway: {}, exit_gateway: {}, specific_details: {}, since: {}",
            self.entry_gateway, self.exit_gateway, self.specific_details, self.since
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
    StoreAccount(oneshot::Sender<Result<(), AccountError>>, String),
    GetAccountSummary(oneshot::Sender<Result<NymVpnAccountSummaryResponse, AccountError>>),
    RegisterDevice(oneshot::Sender<Result<NymVpnDevice, AccountError>>),
    Shutdown,
}

impl fmt::Display for VpnServiceCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnServiceCommand::Connect(_, args) => write!(f, "Connect {{ {args:?} }}"),
            VpnServiceCommand::Disconnect(_) => write!(f, "Disconnect"),
            VpnServiceCommand::Status(_) => write!(f, "Status"),
            VpnServiceCommand::Info(_) => write!(f, "Info"),
            VpnServiceCommand::ImportCredential(_, _) => write!(f, "ImportCredential"),
            VpnServiceCommand::StoreAccount(_, _) => write!(f, "StoreAccount"),
            VpnServiceCommand::GetAccountSummary(_) => write!(f, "GetAccountSummery"),
            VpnServiceCommand::RegisterDevice(_) => write!(f, "GetAccountSummery"),
            VpnServiceCommand::Shutdown => write!(f, "Shutdown"),
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
    pub(crate) min_mixnode_performance: Option<u8>,
    pub(crate) min_gateway_performance: Option<u8>,
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
    pub network_name: String,
    pub endpoints: Vec<nym_vpn_lib::nym_config::defaults::ValidatorDetails>,
    pub nym_vpn_api_url: Option<String>,
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
    pub entry_gateway: NodeIdentity,
    pub exit_gateway: NodeIdentity,
    pub specific_details: ConnectedStateDetails,
    pub since: time::OffsetDateTime,
}

impl fmt::Display for ConnectedResultDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "entry_gateway: {}, exit_gateway: {}, specific_details: {}, since: {}",
            self.entry_gateway, self.exit_gateway, self.specific_details, self.since
        )
    }
}

impl From<VpnConnectedStateDetails> for ConnectedResultDetails {
    fn from(details: VpnConnectedStateDetails) -> Self {
        ConnectedResultDetails {
            entry_gateway: details.entry_gateway,
            exit_gateway: details.exit_gateway,
            specific_details: details.specific_details,
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

pub(crate) struct NymVpnService<S>
where
    S: nym_vpn_store::VpnStorage,
{
    shared_vpn_state: SharedVpnState,

    // Listen for commands from the command interface, like the grpc listener that listens user
    // commands.
    vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,

    // Send commands to the actual vpn service task
    vpn_ctrl_sender: Option<UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>>,

    config_file: PathBuf,

    data_dir: PathBuf,

    // Storage backend
    storage: S,
}

impl NymVpnService<nym_vpn_lib::storage::VpnClientOnDiskStorage> {
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
        let storage = nym_vpn_lib::storage::VpnClientOnDiskStorage::new(data_dir.clone());
        Self {
            shared_vpn_state: SharedVpnState::new(vpn_state_changes_tx),
            vpn_command_rx,
            vpn_ctrl_sender: None,
            config_file,
            data_dir,
            storage,
        }
    }

    pub(crate) async fn init_storage(&self) -> Result<(), ConfigSetupError> {
        // Make sure the data dir exists
        if let Err(err) = create_data_dir(&self.data_dir) {
            self.shared_vpn_state.set(VpnState::NotConnected);
            return Err(err);
        }

        // Generate the device keys if we don't already have them
        if let Err(err) = self.storage.init_keys(None).await {
            self.shared_vpn_state.set(VpnState::NotConnected);
            return Err(ConfigSetupError::FailedToInitKeys { source: err });
        }

        Ok(())
    }
}

impl<S> NymVpnService<S>
where
    S: nym_vpn_store::VpnStorage,
{
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

        info!(
            "Using entry point: {}",
            entry
                .clone()
                .map(|e| e.to_string())
                .unwrap_or("None".to_string())
        );
        info!(
            "Using exit point: {}",
            exit.clone()
                .map(|e| e.to_string())
                .unwrap_or("None".to_string())
        );
        info!("Using options: {:?}", options);

        let config = match self.try_setup_config(entry, exit) {
            Ok(config) => config,
            Err(err) => {
                self.shared_vpn_state.set(VpnState::NotConnected);
                return VpnServiceConnectResult::Fail(err.to_string());
            }
        };

        info!("Using config: {}", config);

        let generic_config = GenericNymVpnConfig {
            mixnet_client_config: MixnetClientConfig {
                enable_poisson_rate: options.enable_poisson_rate,
                disable_background_cover_traffic: options.disable_background_cover_traffic,
                enable_credentials_mode: options.enable_credentials_mode,
                min_mixnode_performance: options.min_mixnode_performance,
                min_gateway_performance: options.min_gateway_performance,
            },
            data_path: Some(self.data_dir.clone()),
            gateway_config: gateway_directory::Config::new_from_env(
                options.min_gateway_performance,
            ),
            entry_point: config.entry_point.clone(),
            exit_point: config.exit_point.clone(),
            nym_ips: None,
            nym_mtu: None,
            dns: options.dns,
            disable_routing: options.disable_routing,
            user_agent: Some(nym_bin_common::bin_info_local_vergen!().into()),
        };

        let nym_vpn = if options.enable_two_hop {
            let mut nym_vpn =
                nym_vpn_lib::NymVpn::new_wireguard_vpn(config.entry_point, config.exit_point);
            nym_vpn.generic_config = generic_config;
            nym_vpn.into()
        } else {
            let mut nym_vpn =
                nym_vpn_lib::NymVpn::new_mixnet_vpn(config.entry_point, config.exit_point);
            nym_vpn.generic_config = generic_config;
            nym_vpn.into()
        };

        let handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(nym_vpn).unwrap();

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
        let network = NymNetworkDetails::new_from_env();
        let bin_info = nym_bin_common::bin_info_local_vergen!();
        VpnServiceInfoResult {
            version: bin_info.build_version.to_string(),
            build_timestamp: time::OffsetDateTime::parse(bin_info.build_timestamp, &Rfc3339).ok(),
            triple: bin_info.cargo_triple.to_string(),
            git_commit: bin_info.commit_sha.to_string(),
            network_name: network.network_name,
            endpoints: network.endpoints,
            nym_vpn_api_url: network.nym_vpn_api_url,
        }
    }

    async fn handle_import_credential(
        &mut self,
        credential: Vec<u8>,
    ) -> Result<Option<OffsetDateTime>, ImportCredentialError> {
        if self.is_running() {
            return Err(ImportCredentialError::VpnRunning);
        }

        let res = import_credential(credential, self.data_dir.clone())
            .await
            .map_err(|err| err.into());
        if res.is_ok()
            && matches!(
                self.shared_vpn_state.get(),
                VpnState::ConnectionFailed(ConnectionFailedError::InvalidCredential {
                    reason: _,
                    location: _,
                    gateway_id: _,
                })
            )
        {
            self.shared_vpn_state.set(VpnState::NotConnected);
        }
        res
    }

    async fn handle_store_account(&mut self, account: String) -> Result<(), AccountError>
    where
        <S as nym_vpn_store::mnemonic::MnemonicStorage>::StorageError: Sync + Send + 'static,
    {
        self.storage
            .store_mnemonic(Mnemonic::parse(&account)?)
            .await
            .map_err(|err| AccountError::FailedToStoreAccount {
                source: Box::new(err),
            })
    }

    async fn load_account(&self) -> Result<VpnApiAccount, AccountError>
    where
        <S as nym_vpn_store::mnemonic::MnemonicStorage>::StorageError: Sync + Send + 'static,
    {
        self.storage
            .load_mnemonic()
            .await
            .map_err(|err| AccountError::FailedToLoadAccount {
                source: Box::new(err),
            })
            .map(VpnApiAccount::from)
    }

    async fn load_device_keys(&self) -> Result<nym_vpn_store::keys::DeviceKeys, AccountError>
    where
        <S as nym_vpn_store::keys::KeyStore>::StorageError: Sync + Send + 'static,
    {
        self.storage
            .load_keys()
            .await
            .map_err(|err| AccountError::FailedToLoadKeys {
                source: Box::new(err),
            })
    }

    async fn handle_get_account_summary(&self) -> Result<NymVpnAccountSummaryResponse, AccountError>
    where
        <S as nym_vpn_store::mnemonic::MnemonicStorage>::StorageError: Sync + Send + 'static,
    {
        // Get account
        let account = self.load_account().await?;

        // Setup client
        let nym_vpn_api_url = get_nym_vpn_api_url()?;
        let user_agent = nym_vpn_lib::UserAgent::from(nym_bin_common::bin_info_local_vergen!());
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client
            .get_account_summary(&account)
            .await
            .map_err(Into::into)
    }

    async fn handle_register_device(&self) -> Result<NymVpnDevice, AccountError>
    where
        <S as nym_vpn_store::mnemonic::MnemonicStorage>::StorageError: Sync + Send + 'static,
        <S as nym_vpn_store::keys::KeyStore>::StorageError: Sync + Send + 'static,
    {
        // Get account
        let account = self.load_account().await?;

        // Get device
        let device_keypair = self.load_device_keys().await?.device_keypair();
        let device = nym_vpn_api_client::types::Device::from(device_keypair);

        // Setup client
        let nym_vpn_api_url = get_nym_vpn_api_url()?;
        let user_agent = nym_vpn_lib::UserAgent::from(nym_bin_common::bin_info_local_vergen!());
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client
            .register_device(&account, &device)
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<()>
    where
        <S as nym_vpn_store::mnemonic::MnemonicStorage>::StorageError: Sync + Send + 'static,
        <S as nym_vpn_store::keys::KeyStore>::StorageError: Sync + Send + 'static,
    {
        while let Some(command) = self.vpn_command_rx.recv().await {
            debug!("VPN: Received command: {command}");
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
                VpnServiceCommand::StoreAccount(tx, account) => {
                    let result = self.handle_store_account(account).await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::GetAccountSummary(tx) => {
                    let result = self.handle_get_account_summary().await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::RegisterDevice(tx) => {
                    let result = self.handle_register_device().await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::Shutdown => {
                    let result = self.handle_disconnect().await;
                    info!("VPN: Shutting down: {:?}", result);
                    while self.is_running() {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                    break;
                }
            }
        }
        Ok(())
    }
}

fn get_nym_vpn_api_url() -> Result<Url, AccountError> {
    NymNetworkDetails::new_from_env()
        .nym_vpn_api_url
        .ok_or(AccountError::MissingApiUrl)?
        .parse()
        .map_err(|_| AccountError::InvalidApiUrl)
}
