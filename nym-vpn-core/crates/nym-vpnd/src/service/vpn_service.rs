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
use time::format_description::well_known::Rfc3339;
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use url::Url;

use nym_vpn_account_controller::{
    AccountCommand, AccountController, AccountState, ReadyToConnect, SharedAccountState,
};
use nym_vpn_api_client::{
    response::{
        NymVpnAccountSummaryResponse, NymVpnDevicesResponse, NymVpnSubscription,
        NymVpnSubscriptionsResponse,
    },
    types::{Percent, VpnApiAccount},
};
use nym_vpn_lib::{
    gateway_directory::{self, EntryPoint, ExitPoint},
    nym_config::defaults::NymNetworkDetails,
    tunnel_state_machine::{
        ConnectionData, DnsOptions, GatewayPerformanceOptions, MixnetEvent, MixnetTunnelOptions,
        NymConfig, TunnelCommand, TunnelConnectionData, TunnelEvent, TunnelSettings, TunnelState,
        TunnelStateMachine, TunnelType,
    },
    MixnetClientConfig, NodeIdentity, Recipient,
};
use nym_vpn_store::keys::KeyStore as _;

use super::{
    config::{
        self, create_config_file, create_data_dir, read_config_file, write_config_file,
        ConfigSetupError, NymVpnServiceConfig, DEFAULT_CONFIG_FILE,
    },
    error::{AccountError, ConnectionFailedError, Error, Result},
};

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

#[allow(clippy::large_enum_variant)]
pub enum VpnServiceCommand {
    Connect(
        oneshot::Sender<VpnServiceConnectResult>,
        ConnectArgs,
        nym_vpn_lib::UserAgent,
    ),
    Disconnect(oneshot::Sender<VpnServiceDisconnectResult>),
    Status(oneshot::Sender<VpnServiceStatusResult>),
    Info(oneshot::Sender<VpnServiceInfoResult>),
    StoreAccount(oneshot::Sender<Result<(), AccountError>>, String),
    IsAccountStored(oneshot::Sender<Result<bool, AccountError>>),
    RemoveAccount(oneshot::Sender<Result<(), AccountError>>),
    GetLocalAccountState(oneshot::Sender<Result<AccountState, AccountError>>),
    GetAccountSummary(oneshot::Sender<Result<NymVpnAccountSummaryResponse, AccountError>>),
    GetDevices(oneshot::Sender<Result<NymVpnDevicesResponse, AccountError>>),
    RegisterDevice(oneshot::Sender<Result<(), AccountError>>),
    RequestZkNym(oneshot::Sender<Result<(), AccountError>>),
    GetDeviceZkNyms(oneshot::Sender<Result<(), AccountError>>),
    GetFreePasses(oneshot::Sender<Result<NymVpnSubscriptionsResponse, AccountError>>),
    ApplyFreepass(
        oneshot::Sender<Result<NymVpnSubscription, AccountError>>,
        String,
    ),
    IsReadyToConnect(oneshot::Sender<Result<ReadyToConnect, AccountError>>),
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
            VpnServiceCommand::StoreAccount(_, _) => write!(f, "StoreAccount"),
            VpnServiceCommand::IsAccountStored(_) => write!(f, "IsAccountStored"),
            VpnServiceCommand::RemoveAccount(_) => write!(f, "RemoveAccount"),
            VpnServiceCommand::GetLocalAccountState(_) => write!(f, "GetLocalAccountState"),
            VpnServiceCommand::GetAccountSummary(_) => write!(f, "GetAccountSummery"),
            VpnServiceCommand::GetDevices(_) => write!(f, "GetDevices"),
            VpnServiceCommand::RegisterDevice(_) => write!(f, "RegisterDevice"),
            VpnServiceCommand::RequestZkNym(_) => write!(f, "RequestZkNym"),
            VpnServiceCommand::GetDeviceZkNyms(_) => write!(f, "GetDeviceZkNyms"),
            VpnServiceCommand::GetFreePasses(_) => write!(f, "GetFreePasses"),
            VpnServiceCommand::ApplyFreepass(_, _) => write!(f, "ApplyFreepass"),
            VpnServiceCommand::IsReadyToConnect(_) => write!(f, "IsReadyToConnect"),
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

#[derive(Debug)]
pub enum VpnServiceConnectResult {
    Success,
    Fail(String),
}

impl VpnServiceConnectResult {
    #[allow(unused)]
    pub fn is_success(&self) -> bool {
        matches!(self, VpnServiceConnectResult::Success)
    }
}

#[derive(Debug)]
pub enum VpnServiceDisconnectResult {
    Success,
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

impl From<ConnectionData> for ConnectedResultDetails {
    fn from(value: ConnectionData) -> Self {
        ConnectedResultDetails {
            entry_gateway: *value.entry_gateway,
            exit_gateway: *value.exit_gateway,
            specific_details: ConnectedStateDetails::from(value.tunnel),
            since: value.connected_at,
        }
    }
}

impl From<TunnelConnectionData> for ConnectedStateDetails {
    fn from(value: TunnelConnectionData) -> Self {
        match value {
            TunnelConnectionData::Mixnet(data) => {
                ConnectedStateDetails::Mix(Box::new(MixConnectedStateDetails {
                    nym_address: *data.nym_address,
                    exit_ipr: *data.exit_ipr,
                    ipv4: data.ipv4,
                    ipv6: data.ipv6,
                }))
            }
            TunnelConnectionData::Wireguard(data) => {
                // FIXME: we must accept ipv6 too in the future!
                ConnectedStateDetails::Wg(WgConnectedStateDetails {
                    entry_ipv4: match data.entry.endpoint.ip() {
                        IpAddr::V4(addr) => addr,
                        IpAddr::V6(_) => Ipv4Addr::LOCALHOST,
                    },
                    exit_ipv4: match data.exit.endpoint.ip() {
                        IpAddr::V4(addr) => addr,
                        IpAddr::V6(_) => Ipv4Addr::LOCALHOST,
                    },
                })
            }
        }
    }
}

impl From<TunnelState> for VpnServiceStatusResult {
    fn from(value: TunnelState) -> Self {
        match value {
            TunnelState::Connected { connection_data } => {
                Self::Connected(Box::new(ConnectedResultDetails {
                    entry_gateway: *connection_data.entry_gateway,
                    exit_gateway: *connection_data.exit_gateway,
                    specific_details: ConnectedStateDetails::from(connection_data.tunnel),
                    since: time::OffsetDateTime::now_utc(),
                }))
            }
            TunnelState::Connecting => Self::Connecting,
            TunnelState::Disconnected => Self::NotConnected,
            TunnelState::Disconnecting { .. } => Self::Disconnecting,
            TunnelState::Error(e) => Self::ConnectionFailed(ConnectionFailedError::InternalError(
                format!("Error state: {:?}", e),
            )),
        }
    }
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

impl VpnServiceStatusResult {
    pub fn error(&self) -> Option<ConnectionFailedError> {
        match self {
            VpnServiceStatusResult::ConnectionFailed(reason) => Some(reason.clone()),
            _ => None,
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

impl From<TunnelState> for VpnServiceStateChange {
    fn from(value: TunnelState) -> Self {
        match value {
            TunnelState::Connecting => Self::Connecting,
            TunnelState::Connected { .. } => Self::Connected,
            TunnelState::Disconnected => Self::NotConnected,
            TunnelState::Disconnecting { .. } => Self::Disconnecting,
            TunnelState::Error(reason) => Self::ConnectionFailed(
                ConnectionFailedError::InternalError(format!("Error state: {:?}", reason)),
            ),
        }
    }
}

impl VpnServiceStateChange {
    pub fn error(&self) -> Option<ConnectionFailedError> {
        match self {
            VpnServiceStateChange::ConnectionFailed(reason) => Some(reason.clone()),
            _ => None,
        }
    }
}

pub(crate) struct NymVpnService<S>
where
    S: nym_vpn_store::VpnStorage,
{
    // The account state, updated by the account controller
    shared_account_state: SharedAccountState,

    // Listen for commands from the command interface, like the grpc listener that listens user
    // commands.
    vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,

    vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
    status_tx: broadcast::Sender<MixnetEvent>,

    #[allow(unused)]
    // Send commands to the account controller
    account_command_tx: tokio::sync::mpsc::UnboundedSender<AccountCommand>,

    config_file: PathBuf,

    // Storage backend
    storage: Arc<tokio::sync::Mutex<S>>,

    // Last known tunnel state.
    tunnel_state: TunnelState,

    // Tunnel state machine handle.
    state_machine_handle: JoinHandle<()>,

    // Command channel for state machine
    command_sender: mpsc::UnboundedSender<TunnelCommand>,

    // Event channel for receiving events from state machine
    event_receiver: mpsc::UnboundedReceiver<TunnelEvent>,

    // Service shutdown token.
    shutdown_token: CancellationToken,
}

impl NymVpnService<nym_vpn_lib::storage::VpnClientOnDiskStorage> {
    pub(crate) fn spawn(
        vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        status_tx: broadcast::Sender<MixnetEvent>,
        shutdown_token: CancellationToken,
    ) -> JoinHandle<()> {
        tracing::info!("Starting VPN service");
        tokio::spawn(async {
            match NymVpnService::new(
                vpn_state_changes_tx,
                vpn_command_rx,
                status_tx,
                shutdown_token,
            )
            .await
            {
                Ok(service) => {
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
        status_tx: broadcast::Sender<MixnetEvent>,
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

        // Make sure the data dir exists
        create_data_dir(&data_dir).map_err(Error::ConfigSetup)?;

        // Generate the device keys if we don't already have them
        storage
            .lock()
            .await
            .init_keys(None)
            .await
            .map_err(|source| Error::ConfigSetup(ConfigSetupError::FailedToInitKeys { source }))?;

        // We need to create the user agent here and not in the controller so that we correctly
        // pick up build time constants.
        let user_agent = crate::util::construct_user_agent();
        let account_controller = AccountController::new(
            Arc::clone(&storage),
            data_dir.clone(),
            user_agent.clone(),
            shutdown_token.child_token(),
        )
        .await;

        let shared_account_state = account_controller.shared_state();
        let account_command_tx = account_controller.command_tx();
        let _account_controller_handle = tokio::task::spawn(account_controller.run());

        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let tunnel_settings = TunnelSettings::default();
        let nym_config = NymConfig {
            data_path: Some(data_dir.clone()),
            gateway_config: gateway_directory::Config::new_from_env(),
        };

        let state_machine_handle = TunnelStateMachine::spawn(
            command_receiver,
            event_sender,
            nym_config,
            tunnel_settings,
            shutdown_token.child_token(),
        )
        .await
        .map_err(Error::StateMachine)?;

        Ok(Self {
            shared_account_state,
            vpn_command_rx,
            vpn_state_changes_tx,
            status_tx,
            account_command_tx,
            config_file,
            storage,
            tunnel_state: TunnelState::Disconnected,
            state_machine_handle,
            command_sender,
            event_receiver,
            shutdown_token,
        })
    }
}

impl<S> NymVpnService<S>
where
    S: nym_vpn_store::VpnStorage,
{
    pub(crate) async fn run(mut self) -> anyhow::Result<()> {
        // Start by refreshing the account state
        self.account_command_tx
            .send(AccountCommand::UpdateSharedAccountState)?;

        loop {
            tokio::select! {
                Some(command) = self.vpn_command_rx.recv() => {
                    tracing::debug!("VPN: Received command: {command}");
                    self.handle_service_command(command).await;
                }
                Some(event) = self.event_receiver.recv() => {
                    tracing::info!("Tunnel event: {:?}", event);
                    match event {
                        TunnelEvent::NewState(new_state) => {
                            let vpn_state_change = VpnServiceStateChange::from(new_state.clone());
                            if let Err(e) = self.vpn_state_changes_tx.send(vpn_state_change) {
                                tracing::error!("Failed to send vpn state change: {}", e);
                            }
                        }
                        TunnelEvent::MixnetEvent(event) => {
                            if let Err(e) = self.status_tx.send(event) {
                                tracing::error!("Failed to send mixnet event: {}", e);
                            }
                        }
                    }
                }
                _ = self.shutdown_token.cancelled() => {
                    tracing::info!("Received shutdown signal");
                    break;
                }
                else => {
                    tracing::warn!("Event loop is interrupted");
                    break;
                }
            }
        }

        tracing::info!("Exiting vpn service run loop");

        if let Err(e) = self.state_machine_handle.await {
            tracing::error!("Failed to join on state machine handle: {}", e);
        }

        Ok(())
    }

    async fn handle_service_command(&mut self, command: VpnServiceCommand) {
        match command {
            VpnServiceCommand::Connect(tx, connect_args, user_agent) => {
                let result = self.handle_connect(connect_args, user_agent).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::Disconnect(tx) => {
                let result = self.handle_disconnect().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::Status(tx) => {
                let result = self.handle_status().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::Info(tx) => {
                let result = self.handle_info().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::StoreAccount(tx, account) => {
                let result = self.handle_store_account(account).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::IsAccountStored(tx) => {
                let result = self.handle_is_account_stored().await;
                tx.send(result).unwrap();
            }
            VpnServiceCommand::RemoveAccount(tx) => {
                let result = self.handle_remove_account().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetLocalAccountState(tx) => {
                let result = self.handle_get_local_account_state().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetAccountSummary(tx) => {
                let result = self.handle_get_account_summary().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetDevices(tx) => {
                let result = self.handle_get_devices().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::RegisterDevice(tx) => {
                let result = self.handle_register_device().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::RequestZkNym(tx) => {
                let result = self.handle_request_zk_nym().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetDeviceZkNyms(tx) => {
                let result = self.handle_get_device_zk_nyms().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetFreePasses(tx) => {
                let result = self.handle_get_free_passes().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::ApplyFreepass(tx, code) => {
                let result = self.handle_apply_freepass(code).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::IsReadyToConnect(tx) => {
                let result = Ok(self.handle_is_ready_to_connect().await);
                tx.send(result).unwrap();
            }
        }
    }

    fn try_setup_config(
        &self,
        entry: Option<gateway_directory::EntryPoint>,
        exit: Option<gateway_directory::ExitPoint>,
    ) -> Result<NymVpnServiceConfig> {
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
            write_config_file(&self.config_file, &read_config).map_err(Error::ConfigSetup)?;
            read_config
        } else {
            let config = NymVpnServiceConfig {
                entry_point: entry.unwrap_or(EntryPoint::Random),
                exit_point: exit.unwrap_or(ExitPoint::Random),
            };
            create_config_file(&self.config_file, config).map_err(Error::ConfigSetup)?
        };
        Ok(config)
    }

    async fn handle_connect(
        &mut self,
        connect_args: ConnectArgs,
        _user_agent: nym_vpn_lib::UserAgent, // todo: use user-agent!
    ) -> VpnServiceConnectResult {
        match self.shared_account_state.is_ready_to_connect().await {
            ReadyToConnect::Ready => {}
            not_ready_to_connect => {
                tracing::info!("Not ready to connect: {:?}", not_ready_to_connect);
                return VpnServiceConnectResult::Fail(format!("{:?}", not_ready_to_connect));
            }
        }

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

        let config = match self.try_setup_config(entry, exit) {
            Ok(config) => config,
            Err(err) => {
                return VpnServiceConnectResult::Fail(err.to_string());
            }
        };
        tracing::info!("Using config: {}", config);

        let gateway_options = GatewayPerformanceOptions {
            mixnet_min_performance: options
                .min_gateway_mixnet_performance
                .map(|x| x.round_to_integer()),
            vpn_min_performance: options
                .min_gateway_vpn_performance
                .map(|x| x.round_to_integer()),
        };

        let mixnet_client_config = MixnetClientConfig {
            enable_poisson_rate: options.enable_poisson_rate,
            disable_background_cover_traffic: options.disable_background_cover_traffic,
            enable_credentials_mode: options.enable_credentials_mode,
            min_mixnode_performance: options
                .min_mixnode_performance
                .map(|p| p.round_to_integer()),
            min_gateway_performance: options
                .min_gateway_mixnet_performance
                .map(|p| p.round_to_integer()),
        };

        let tunnel_type = if options.enable_two_hop {
            TunnelType::Wireguard
        } else {
            TunnelType::Mixnet
        };

        let dns = options
            .dns
            .map(|addr| DnsOptions::Custom(vec![addr]))
            .unwrap_or(DnsOptions::default());

        let tunnel_settings = TunnelSettings {
            tunnel_type,
            mixnet_tunnel_options: MixnetTunnelOptions::default(),
            gateway_performance_options: gateway_options,
            mixnet_client_config: Some(mixnet_client_config),
            entry_point: Box::new(config.entry_point),
            exit_point: Box::new(config.exit_point),
            dns,
        };

        match self
            .command_sender
            .send(TunnelCommand::SetTunnelSettings(tunnel_settings))
        {
            Ok(()) => self
                .command_sender
                .send(TunnelCommand::Connect)
                .err()
                .map(|e| {
                    tracing::error!("Failed to send command to connect: {}", e);
                    VpnServiceConnectResult::Fail("Internal error".to_owned())
                })
                .unwrap_or(VpnServiceConnectResult::Success),
            Err(e) => {
                tracing::error!("Failed to send command to set tunnel options: {}", e);
                VpnServiceConnectResult::Fail("Internal error".to_owned())
            }
        }
    }

    async fn handle_disconnect(&mut self) -> VpnServiceDisconnectResult {
        self.command_sender
            .send(TunnelCommand::Disconnect)
            .err()
            .map(|e| {
                tracing::error!("Failed to send command to disconnect: {}", e);
                VpnServiceDisconnectResult::Fail("Internal error".to_owned())
            })
            .unwrap_or(VpnServiceDisconnectResult::Success)
    }

    async fn handle_status(&self) -> VpnServiceStatusResult {
        VpnServiceStatusResult::from(self.tunnel_state.clone())
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

    async fn handle_store_account(&mut self, account: String) -> Result<(), AccountError> {
        self.storage
            .lock()
            .await
            .store_mnemonic(Mnemonic::parse(&account)?)
            .await
            .map_err(|err| AccountError::FailedToStoreAccount {
                source: Box::new(err),
            })?;

        self.account_command_tx
            .send(AccountCommand::UpdateSharedAccountState)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })?;

        Ok(())
    }

    async fn handle_is_account_stored(&self) -> Result<bool, AccountError> {
        self.storage
            .lock()
            .await
            .is_mnemonic_stored()
            .await
            .map_err(|err| AccountError::FailedToCheckIfAccountIsStored {
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
            })?;

        self.account_command_tx
            .send(AccountCommand::UpdateSharedAccountState)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })?;

        Ok(())
    }

    async fn handle_get_local_account_state(&self) -> Result<AccountState, AccountError> {
        Ok(self.shared_account_state.lock().await.clone())
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

    #[allow(unused)]
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

    async fn handle_register_device(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::RegisterDevice)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })
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

    async fn handle_request_zk_nym(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::RequestZkNym)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })
    }

    async fn handle_get_device_zk_nyms(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::GetDeviceZkNym)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })
    }

    async fn handle_is_ready_to_connect(&self) -> ReadyToConnect {
        self.shared_account_state.is_ready_to_connect().await
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
