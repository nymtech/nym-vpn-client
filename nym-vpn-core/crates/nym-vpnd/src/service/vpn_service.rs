// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::Arc,
};

use bip39::Mnemonic;
use serde::{Deserialize, Serialize};
use time::{ext, format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use url::Url;

use nym_vpn_account_controller::{AccountCommand, AccountController, SharedAccountState};
use nym_vpn_api_client::{
    response::{
        NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnDevicesResponse, NymVpnSubscription,
        NymVpnSubscriptionsResponse, NymVpnZkNym, NymVpnZkNymResponse,
    },
    types::{GatewayMinPerformance, Percent, VpnApiAccount},
};
use nym_vpn_lib::{
    credentials::import_credential,
    gateway_directory::{self, EntryPoint, ExitPoint},
    nym_config::defaults::NymNetworkDetails,
    tunnel_state_machine::{TunnelCommand, TunnelState, TunnelStateMachine},
    GenericNymVpnConfig, MixnetClientConfig, NodeIdentity, Recipient,
};
use nym_vpn_store::keys::KeyStore as _;

use super::{
    config::{
        self, create_config_file, create_data_dir, read_config_file, write_config_file,
        ConfigSetupError, NymVpnServiceConfig, DEFAULT_CONFIG_FILE,
    },
    error::{AccountError, ConnectionFailedError, ImportCredentialError},
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
    Connect(ConnectArgs, nym_vpn_lib::UserAgent),
    Disconnect,
    Status(oneshot::Sender<VpnServiceStatusResult>),
    Info(oneshot::Sender<VpnServiceInfoResult>),
    ImportCredential(
        oneshot::Sender<Result<Option<OffsetDateTime>, ImportCredentialError>>,
        Vec<u8>,
    ),
    StoreAccount(oneshot::Sender<Result<(), AccountError>>, String),
    RemoveAccount(oneshot::Sender<Result<(), AccountError>>),
    GetAccountSummary(oneshot::Sender<Result<NymVpnAccountSummaryResponse, AccountError>>),
    GetDevices(oneshot::Sender<Result<NymVpnDevicesResponse, AccountError>>),
    RegisterDevice(oneshot::Sender<Result<NymVpnDevice, AccountError>>),
    RequestZkNym(oneshot::Sender<Result<NymVpnZkNym, AccountError>>),
    GetDeviceZkNyms(oneshot::Sender<Result<NymVpnZkNymResponse, AccountError>>),
    GetFreePasses(oneshot::Sender<Result<NymVpnSubscriptionsResponse, AccountError>>),
    ApplyFreepass(
        oneshot::Sender<Result<NymVpnSubscription, AccountError>>,
        String,
    ),
}

impl fmt::Display for VpnServiceCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnServiceCommand::Connect(_, args, user_agent) => {
                write!(f, "Connect {{ {args:?}, {user_agent:?} }}")
            }
            VpnServiceCommand::Disconnect(_) => write!(f, "Disconnect"),
            VpnServiceCommand::Status(_) => write!(f, "Status"),
            VpnServiceCommand::Info(_) => write!(f, "Info"),
            VpnServiceCommand::ImportCredential(_, _) => write!(f, "ImportCredential"),
            VpnServiceCommand::StoreAccount(_, _) => write!(f, "StoreAccount"),
            VpnServiceCommand::RemoveAccount(_) => write!(f, "RemoveAccount"),
            VpnServiceCommand::GetAccountSummary(_) => write!(f, "GetAccountSummery"),
            VpnServiceCommand::GetDevices(_) => write!(f, "GetDevices"),
            VpnServiceCommand::RegisterDevice(_) => write!(f, "RegisterDevice"),
            VpnServiceCommand::RequestZkNym(_) => write!(f, "RequestZkNym"),
            VpnServiceCommand::GetDeviceZkNyms(_) => write!(f, "GetDeviceZkNyms"),
            VpnServiceCommand::GetFreePasses(_) => write!(f, "GetFreePasses"),
            VpnServiceCommand::ApplyFreepass(_, _) => write!(f, "ApplyFreepass"),
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
    pub(crate) min_mixnode_performance: Option<Percent>,
    pub(crate) min_gateway_mixnet_performance: Option<Percent>,
    pub(crate) min_gateway_vpn_performance: Option<Percent>,
    // Consider adding this here once UserAgent implements Serialize/Deserialize
    // pub(crate) user_agent: Option<nym_vpn_lib::UserAgent>,
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
    pub platform: String,
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

pub(crate) struct NymVpnService<S>
where
    S: nym_vpn_store::VpnStorage,
{
    // The account state, updated by the account controller
    #[allow(unused)]
    shared_account_state: SharedAccountState,

    // Listen for commands from the command interface, like the grpc listener that listens user
    // commands.
    vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,

    #[allow(unused)]
    // Send commands to the account controller
    account_command_tx: tokio::sync::mpsc::UnboundedSender<AccountCommand>,

    config_file: PathBuf,

    data_dir: PathBuf,

    // Storage backend
    storage: Arc<tokio::sync::Mutex<S>>,

    state_machine_handle: JoinHandle<()>,
    command_sender: mpsc::UnboundedSender<TunnelCommand>,
    shutdown_token: CancellationToken,
}

impl NymVpnService<nym_vpn_lib::storage::VpnClientOnDiskStorage> {
    pub(crate) fn spawn(
        vpn_state_changes_tx: tokio::sync::broadcast::Sender<VpnServiceStateChange>,
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        shutdown_token: CancellationToken,
    ) -> JoinHandle<()> {
        tracing::info!("Starting VPN service");
        tokio::spawn(async {
            let service = NymVpnService::new(vpn_state_changes_tx, vpn_command_rx, shutdown_token);
            match service.init_storage().await {
                Ok(()) => {
                    tracing::info!("VPN service initialized successfully");

                    match service.run().await {
                        Ok(_) => {
                            tracing::info!("VPN service has successfully exited");
                        }
                        Err(e) => {
                            tracing::error!("VPN service has exited with error: {:?}", e);
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("Failed to initialize VPN service: {:?}", err);
                }
            }
        })
    }

    pub(crate) async fn new(
        vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        shutdown_token: CancellationToken,
    ) -> Result<Self> {
        let config_dir = std::env::var("NYM_VPND_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| config::default_config_dir());
        let config_file = config_dir.join(DEFAULT_CONFIG_FILE);
        let data_dir = std::env::var("NYM_VPND_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| config::default_data_dir());

        let storage = Arc::new(tokio::sync::Mutex::new(
            nym_vpn_lib::storage::VpnClientOnDiskStorage::new(data_dir.clone()),
        ));

        // We need to create the user agent here and not in the controller so that we correctly
        // pick up build time constants.
        let user_agent = crate::util::construct_user_agent();
        let account_controller =
            AccountController::new(Arc::clone(&storage), user_agent.clone(), shutdown_token);
        let shared_account_state = account_controller.shared_state();
        let account_command_tx = account_controller.command_tx();
        let _account_controller_handle = tokio::task::spawn(account_controller.run());

        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // TODO: rework this.
        let config = GenericNymVpnConfig {
            mixnet_client_config: MixnetClientConfig::wireguard_default(),
            data_path: Some(data_dir.clone()),
            gateway_config: gateway_directory::Config::default(),
            entry_point: EntryPoint::Random,
            exit_point: ExitPoint::Random,
            nym_ips: None,
            nym_mtu: None,
            dns: None,
            disable_routing: false,
            user_agent: Some(user_agent),
        };

        let state_machine_handle = TunnelStateMachine::spawn(
            command_receiver,
            event_sender,
            config,
            true,
            shutdown_token.child_token(),
        )
        .await?;

        Self {
            shared_account_state,
            vpn_command_rx,
            account_command_tx,
            config_file,
            data_dir,
            storage,
            state_machine_handle,
            command_sender,
            shutdown_token,
        }
    }

    pub(crate) async fn init_storage(&self) -> Result<(), ConfigSetupError> {
        // Make sure the data dir exists
        if let Err(err) = create_data_dir(&self.data_dir) {
            return Err(err);
        }

        // Generate the device keys if we don't already have them
        if let Err(err) = self.storage.lock().await.init_keys(None).await {
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
                    tracing::error!(
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

    async fn handle_connect(
        &mut self,
        connect_args: ConnectArgs,
        user_agent: nym_vpn_lib::UserAgent,
    ) {
        let ConnectArgs {
            entry,
            exit,
            options,
        } = connect_args;

        tracing::info!(
            "Using entry point: {}",
            entry
                .clone()
                .map(|e| e.to_string())
                .unwrap_or("None".to_string())
        );
        tracing::info!(
            "Using exit point: {}",
            exit.clone()
                .map(|e| e.to_string())
                .unwrap_or("None".to_string())
        );
        tracing::info!("Using options: {:?}", options);

        let config = NymVpnServiceConfig {
            entry_point: EntryPoint::Random,
            exit_point: ExitPoint::Random,
        };

        // TODO: set entry/exit & persist config via separate call.

        tracing::info!("Using config: {}", config);

        let min_gateway_performance = GatewayMinPerformance {
            mixnet_min_performance: options.min_gateway_mixnet_performance,
            vpn_min_performance: options.min_gateway_vpn_performance,
        };

        let gateway_config = gateway_directory::Config::new_from_env()
            .with_min_gateway_performance(min_gateway_performance);

        let generic_config = GenericNymVpnConfig {
            mixnet_client_config: MixnetClientConfig {
                enable_poisson_rate: options.enable_poisson_rate,
                disable_background_cover_traffic: options.disable_background_cover_traffic,
                enable_credentials_mode: options.enable_credentials_mode,
                min_mixnode_performance: options
                    .min_mixnode_performance
                    .map(|p| p.round_to_integer()),
                min_gateway_performance: options
                    .min_gateway_mixnet_performance
                    .map(|p| p.round_to_integer()),
            },
            data_path: Some(self.data_dir.clone()),
            gateway_config,
            entry_point: config.entry_point.clone(),
            exit_point: config.exit_point.clone(),
            nym_ips: None,
            nym_mtu: None,
            dns: options.dns,
            disable_routing: options.disable_routing,
            user_agent: Some(user_agent),
        };

        if let Err(e) = self.command_sender.send(TunnelCommand::Connect) {
            tracing::error!("Failed to send a connect command: {}", e);
        }
    }

    fn is_running(&self) -> bool {
        self.vpn_ctrl_sender
            .as_ref()
            .map(|s| !s.is_closed())
            .unwrap_or(false)
    }

    async fn handle_disconnect(&mut self) {
        if let Err(e) = self.command_sender.send(TunnelCommand::Disconnect) {
            tracing::error!("Failed to send command to disconnect: {}", e);
        }
    }

    async fn handle_status(&self) -> VpnServiceStatusResult {
        self.shared_vpn_state.get().into()
    }

    async fn handle_info(&self) -> VpnServiceInfoResult {
        let network = NymNetworkDetails::new_from_env();
        let bin_info = nym_bin_common::bin_info_local_vergen!();
        let user_agent = crate::util::construct_user_agent();

        VpnServiceInfoResult {
            version: bin_info.build_version.to_string(),
            build_timestamp: time::OffsetDateTime::parse(bin_info.build_timestamp, &Rfc3339).ok(),
            triple: bin_info.cargo_triple.to_string(),
            platform: user_agent.platform,
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

    async fn handle_store_account(&mut self, account: String) -> Result<(), AccountError> {
        self.storage
            .lock()
            .await
            .store_mnemonic(Mnemonic::parse(&account)?)
            .await
            .map_err(|err| AccountError::FailedToStoreAccount {
                source: Box::new(err),
            })
    }

    async fn handle_remove_account(&mut self) -> Result<(), AccountError> {
        self.storage
            .lock()
            .await
            .remove_mnemonic()
            .await
            .map_err(|err| AccountError::FailedToRemoveAccount {
                source: Box::new(err),
            })
    }

    async fn load_account(&self) -> Result<VpnApiAccount, AccountError> {
        self.storage
            .lock()
            .await
            .load_mnemonic()
            .await
            .map_err(|err| AccountError::FailedToLoadAccount {
                source: Box::new(err),
            })
            .map(VpnApiAccount::from)
            .inspect(|account| tracing::info!("Loading account id: {}", account.id()))
    }

    async fn load_device_keys(&self) -> Result<nym_vpn_store::keys::DeviceKeys, AccountError> {
        self.storage
            .lock()
            .await
            .load_keys()
            .await
            .map_err(|err| AccountError::FailedToLoadKeys {
                source: Box::new(err),
            })
            .inspect(|keys| {
                let device_keypair = keys.device_keypair();
                tracing::info!("Loading device key: {}", device_keypair.public_key())
            })
    }

    async fn handle_get_account_summary(
        &self,
    ) -> Result<NymVpnAccountSummaryResponse, AccountError> {
        // Get account
        let account = self.load_account().await?;

        // Setup client
        let nym_vpn_api_url = get_nym_vpn_api_url()?;
        let user_agent = crate::util::construct_user_agent();
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client
            .get_account_summary(&account)
            .await
            .map_err(Into::into)
    }

    async fn handle_get_devices(&self) -> Result<NymVpnDevicesResponse, AccountError> {
        // Get account
        let account = self.load_account().await?;

        // Setup client
        let nym_vpn_api_url = get_nym_vpn_api_url()?;
        let user_agent = crate::util::construct_user_agent();
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client.get_devices(&account).await.map_err(Into::into)
    }

    async fn handle_register_device(&self) -> Result<NymVpnDevice, AccountError> {
        // Get account
        let account = self.load_account().await?;

        // Get device
        let device_keypair = self.load_device_keys().await?.device_keypair();
        let device = nym_vpn_api_client::types::Device::from(device_keypair);

        // Setup client
        let nym_vpn_api_url = get_nym_vpn_api_url()?;
        let user_agent = crate::util::construct_user_agent();
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client
            .register_device(&account, &device)
            .await
            .map_err(Into::into)
    }

    async fn handle_get_free_passes(&self) -> Result<NymVpnSubscriptionsResponse, AccountError> {
        // Get account
        let account = self.load_account().await?;

        // Setup client
        let nym_vpn_api_url = get_nym_vpn_api_url()?;
        let user_agent = crate::util::construct_user_agent();
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client
            .get_free_passes(&account)
            .await
            .map_err(Into::into)
    }

    async fn handle_apply_freepass(
        &self,
        code: String,
    ) -> Result<NymVpnSubscription, AccountError> {
        // Get account
        let account = self.load_account().await?;

        // Setup client
        let nym_vpn_api_url = get_nym_vpn_api_url()?;
        let user_agent = crate::util::construct_user_agent();
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client
            .apply_freepass(&account, code)
            .await
            .map_err(Into::into)
    }

    async fn handle_request_zk_nym(&self) -> Result<NymVpnZkNym, AccountError> {
        // Get account
        let account = self.load_account().await?;

        // Get device
        let device_keypair = self.load_device_keys().await?.device_keypair();
        let device = nym_vpn_api_client::types::Device::from(device_keypair);

        // Setup client
        let nym_vpn_api_url = get_nym_vpn_api_url()?;
        let user_agent = crate::util::construct_user_agent();
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client
            .request_zk_nym(&account, &device)
            .await
            .map_err(Into::into)
    }

    async fn handle_get_device_zk_nyms(&self) -> Result<NymVpnZkNymResponse, AccountError> {
        // Get account
        let account = self.load_account().await?;

        // Get device
        let device_keypair = self.load_device_keys().await?.device_keypair();
        let device = nym_vpn_api_client::types::Device::from(device_keypair);

        // Setup client
        let nym_vpn_api_url = get_nym_vpn_api_url()?;
        let user_agent = crate::util::construct_user_agent();
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client
            .get_device_zk_nyms(&account, &device)
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<()> {
        // Start by refreshing the account state
        self.account_command_tx
            .send(AccountCommand::RefreshAccountState)?;

        loop {
            tokio::select! {
                Some(command) = self.vpn_command_rx.recv() => {
                    tracing::debug!("VPN: Received command: {command}");
                    match command {
                        VpnServiceCommand::Connect(tx, connect_args, user_agent) => {
                            let result = self.handle_connect(connect_args, user_agent).await;
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
                        VpnServiceCommand::RemoveAccount(tx) => {
                            let result = self.handle_remove_account().await;
                            tx.send(result).unwrap();
                        }
                        VpnServiceCommand::GetAccountSummary(tx) => {
                            let result = self.handle_get_account_summary().await;
                            tx.send(result).unwrap();
                        }
                        VpnServiceCommand::GetDevices(tx) => {
                            let result = self.handle_get_devices().await;
                            tx.send(result).unwrap();
                        }
                        VpnServiceCommand::RegisterDevice(tx) => {
                            let result = self.handle_register_device().await;
                            tx.send(result).unwrap();
                        }
                        VpnServiceCommand::RequestZkNym(tx) => {
                            let result = self.handle_request_zk_nym().await;
                            tx.send(result).unwrap();
                        }
                        VpnServiceCommand::GetDeviceZkNyms(tx) => {
                            let result = self.handle_get_device_zk_nyms().await;
                            tx.send(result).unwrap();
                        }
                        VpnServiceCommand::GetFreePasses(tx) => {
                            let result = self.handle_get_free_passes().await;
                            tx.send(result).unwrap();
                        }
                        VpnServiceCommand::ApplyFreepass(tx, code) => {
                            let result = self.handle_apply_freepass(code).await;
                            tx.send(result).unwrap();
                        }
                    }
                },
                _ = self.shutdown_token.cancelled() => {
                    tracing::info!("Received shutdown signal");
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
        .inspect(|url| tracing::info!("Using nym-vpn-api url: {}", url))
}
