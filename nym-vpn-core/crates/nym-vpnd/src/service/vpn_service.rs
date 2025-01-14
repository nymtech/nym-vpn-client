// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::Arc,
};

use bip39::Mnemonic;
use futures::FutureExt;
use nym_vpn_network_config::{
    FeatureFlags, Network, NymNetwork, NymVpnNetwork, ParsedAccountLinks, SystemMessages,
};
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use nym_vpn_account_controller::{
    AccountCommand, AccountCommandError, AccountController, AccountControllerCommander,
    AccountStateSummary, AvailableTicketbooks, ReadyToConnect, SharedAccountState,
};
use nym_vpn_api_client::{
    response::{NymVpnDevice, NymVpnUsage},
    types::Percent,
};
use nym_vpn_lib::{
    gateway_directory::{self, EntryPoint, ExitPoint},
    tunnel_state_machine::{
        DnsOptions, GatewayPerformanceOptions, MixnetTunnelOptions, NymConfig, TunnelCommand,
        TunnelSettings, TunnelStateMachine, WireguardMultihopMode, WireguardTunnelOptions,
    },
    MixnetClientConfig, NodeIdentity, Recipient, UserAgent,
};
use nym_vpn_lib_types::{
    ConnectionData, TunnelConnectionData, TunnelEvent, TunnelState, TunnelType,
};
use zeroize::Zeroizing;

use crate::{config::GlobalConfigFile, service::AccountNotReady};

use super::{
    config::{ConfigSetupError, NetworkEnvironments, NymVpnServiceConfig, DEFAULT_CONFIG_FILE},
    error::{AccountError, ConnectionFailedError, Error, Result, SetNetworkError},
    VpnServiceConnectError, VpnServiceDisconnectError,
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

// Seed used to generate device identity keys
type Seed = [u8; 32];

type Locale = String;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, strum::Display)]
pub enum VpnServiceCommand {
    Info(oneshot::Sender<VpnServiceInfo>, ()),
    SetNetwork(oneshot::Sender<Result<(), SetNetworkError>>, String),
    GetSystemMessages(oneshot::Sender<SystemMessages>, ()),
    GetFeatureFlags(oneshot::Sender<Option<FeatureFlags>>, ()),
    Connect(
        oneshot::Sender<Result<(), VpnServiceConnectError>>,
        ConnectArgs,
    ),
    Disconnect(oneshot::Sender<Result<(), VpnServiceDisconnectError>>, ()),
    Status(oneshot::Sender<VpnServiceStatus>, ()),
    StoreAccount(oneshot::Sender<Result<(), AccountError>>, Zeroizing<String>),
    IsAccountStored(oneshot::Sender<Result<bool, AccountError>>, ()),
    ForgetAccount(oneshot::Sender<Result<(), AccountError>>, ()),
    GetAccountIdentity(oneshot::Sender<Result<Option<String>, AccountError>>, ()),
    GetAccountLinks(
        oneshot::Sender<Result<ParsedAccountLinks, AccountError>>,
        Locale,
    ),
    GetAccountState(
        oneshot::Sender<Result<AccountStateSummary, AccountError>>,
        (),
    ),
    RefreshAccountState(oneshot::Sender<Result<(), AccountError>>, ()),
    GetAccountUsage(oneshot::Sender<Result<Vec<NymVpnUsage>, AccountError>>, ()),
    IsReadyToConnect(oneshot::Sender<Result<ReadyToConnect, AccountError>>, ()),
    ResetDeviceIdentity(oneshot::Sender<Result<(), AccountError>>, Option<Seed>),
    GetDeviceIdentity(oneshot::Sender<Result<String, AccountError>>, ()),
    RegisterDevice(oneshot::Sender<Result<(), AccountError>>, ()),
    GetDevices(oneshot::Sender<Result<Vec<NymVpnDevice>, AccountError>>, ()),
    GetActiveDevices(oneshot::Sender<Result<Vec<NymVpnDevice>, AccountError>>, ()),
    RequestZkNym(oneshot::Sender<Result<(), AccountError>>, ()),
    GetDeviceZkNyms(oneshot::Sender<Result<(), AccountError>>, ()),
    GetZkNymsAvailableForDownload(oneshot::Sender<Result<(), AccountError>>, ()),
    GetZkNymById(oneshot::Sender<Result<(), AccountError>>, String),
    ConfirmZkNymIdDownloaded(oneshot::Sender<Result<(), AccountError>>, String),
    GetAvailableTickets(
        oneshot::Sender<Result<AvailableTicketbooks, AccountError>>,
        (),
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
    pub(crate) netstack: bool,
    pub(crate) disable_poisson_rate: bool,
    pub(crate) disable_background_cover_traffic: bool,
    pub(crate) enable_credentials_mode: bool,
    pub(crate) min_mixnode_performance: Option<Percent>,
    pub(crate) min_gateway_mixnet_performance: Option<Percent>,
    pub(crate) min_gateway_vpn_performance: Option<Percent>,
    pub(crate) user_agent: Option<UserAgent>,
}

// Respond with the current state of the VPN service. This is currently almost the same as VpnState,
// but it's conceptually not the same thing, so we keep them separate.
#[derive(Clone, Debug)]
pub enum VpnServiceStatus {
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
            // FIXME: this cannot be mapped correctly
            since: value.connected_at.unwrap_or(OffsetDateTime::now_utc()),
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

impl From<TunnelState> for VpnServiceStatus {
    fn from(value: TunnelState) -> Self {
        match value {
            TunnelState::Connected { connection_data } => {
                Self::Connected(Box::new(ConnectedResultDetails {
                    entry_gateway: *connection_data.entry_gateway,
                    exit_gateway: *connection_data.exit_gateway,
                    specific_details: ConnectedStateDetails::from(connection_data.tunnel),
                    // FIXME: impossible to map this correctly
                    since: connection_data
                        .connected_at
                        .unwrap_or(OffsetDateTime::now_utc()),
                }))
            }
            TunnelState::Connecting { .. } => Self::Connecting,
            TunnelState::Disconnected | TunnelState::Offline { .. } => Self::NotConnected,
            TunnelState::Disconnecting { .. } => Self::Disconnecting,
            TunnelState::Error(e) => Self::ConnectionFailed(ConnectionFailedError::InternalError(
                format!("Error state: {:?}", e),
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VpnServiceInfo {
    pub version: String,
    pub build_timestamp: Option<time::OffsetDateTime>,
    pub triple: String,
    pub platform: String,
    pub git_commit: String,
    pub nym_network: NymNetwork,
    pub nym_vpn_network: NymVpnNetwork,
}

impl fmt::Display for VpnServiceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnServiceStatus::NotConnected => write!(f, "NotConnected"),
            VpnServiceStatus::Connecting => write!(f, "Connecting"),
            VpnServiceStatus::Connected(details) => write!(f, "Connected({})", details),
            VpnServiceStatus::Disconnecting => write!(f, "Disconnecting"),
            VpnServiceStatus::ConnectionFailed(reason) => {
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

impl VpnServiceStatus {
    pub fn error(&self) -> Option<ConnectionFailedError> {
        match self {
            VpnServiceStatus::ConnectionFailed(reason) => Some(reason.clone()),
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
            TunnelState::Connecting { .. } => Self::Connecting,
            TunnelState::Connected { .. } => Self::Connected,
            TunnelState::Disconnected | TunnelState::Offline { .. } => Self::NotConnected,
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
    // The network environment
    network_env: Network,

    // The user agent used for HTTP request
    user_agent: UserAgent,

    // The account state, updated by the account controller
    shared_account_state: SharedAccountState,

    // Listen for commands from the command interface, like the grpc listener that listens user
    // commands.
    vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,

    // Broadcast channel for sending state changes to the outside world
    vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,

    // Broadcast channel for sending tunnel events to the outside world
    tunnel_event_tx: broadcast::Sender<TunnelEvent>,

    // Send commands to the account controller
    account_command_tx: AccountControllerCommander,

    // Path to the main config file
    config_file: PathBuf,

    // Path to the data directory
    data_dir: PathBuf,

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

    // The (optional) recipient to send statistics to
    statistics_recipient: Option<Recipient>,
}

impl NymVpnService<nym_vpn_lib::storage::VpnClientOnDiskStorage> {
    pub(crate) fn spawn(
        vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        tunnel_event_tx: broadcast::Sender<TunnelEvent>,
        shutdown_token: CancellationToken,
        network_env: Network,
        user_agent: UserAgent,
    ) -> JoinHandle<()> {
        tracing::info!("Starting VPN service");
        tokio::spawn(async {
            match NymVpnService::new(
                vpn_state_changes_tx,
                vpn_command_rx,
                tunnel_event_tx,
                shutdown_token,
                network_env,
                user_agent,
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
        tunnel_event_tx: broadcast::Sender<TunnelEvent>,
        shutdown_token: CancellationToken,
        network_env: Network,
        user_agent: UserAgent,
    ) -> Result<Self> {
        let network_name = network_env.nym_network_details().network_name.clone();

        let config_dir = super::config::config_dir().join(&network_name);
        let config_file = config_dir.join(DEFAULT_CONFIG_FILE);
        let data_dir = super::config::data_dir().join(&network_name);

        let storage = Arc::new(tokio::sync::Mutex::new(
            nym_vpn_lib::storage::VpnClientOnDiskStorage::new(data_dir.clone()),
        ));

        // Make sure the data dir exists
        super::config::create_data_dir(&data_dir).map_err(Error::ConfigSetup)?;

        let statistics_recipient = network_env.get_feature_flag_stats_recipient();

        let account_controller = AccountController::new(
            Arc::clone(&storage),
            data_dir.clone(),
            user_agent.clone(),
            None,
            network_env.clone(),
            shutdown_token.child_token(),
        )
        .await
        .map_err(|source| Error::Account(AccountError::AccountControllerError { source }))?;

        // These are used to interact with the account controller
        let shared_account_state = account_controller.shared_state();
        let account_command_tx = account_controller.commander();
        let _account_controller_handle = tokio::task::spawn(account_controller.run());

        // These used to interact with the tunnel state machine
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let tunnel_settings = TunnelSettings::default();
        let api_url = network_env
            .api_url()
            .ok_or(Error::ConfigSetup(ConfigSetupError::MissingApiUrl))?;
        let gateway_config = gateway_directory::Config {
            api_url,
            nym_vpn_api_url: Some(network_env.vpn_api_url()),
            min_gateway_performance: None,
        };
        let nym_config = NymConfig {
            data_path: Some(data_dir.clone()),
            gateway_config,
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
            network_env,
            user_agent,
            shared_account_state,
            vpn_command_rx,
            vpn_state_changes_tx,
            tunnel_event_tx,
            account_command_tx,
            config_file,
            data_dir,
            storage,
            tunnel_state: TunnelState::Disconnected,
            state_machine_handle,
            command_sender,
            event_receiver,
            shutdown_token,
            statistics_recipient,
        })
    }
}

impl<S> NymVpnService<S>
where
    S: nym_vpn_store::VpnStorage,
{
    pub(crate) async fn run(mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                Some(command) = self.vpn_command_rx.recv() => {
                    tracing::debug!("VPN: Received command: {command}");
                    self.handle_service_command(command).await;
                }
                Some(event) = self.event_receiver.recv() => {
                    if let Err(e) = self.tunnel_event_tx.send(event.clone()) {
                        tracing::error!("Failed to send tunnel event: {}", e);
                    }

                    match event {
                        TunnelEvent::NewState(new_state) => {
                            self.tunnel_state = new_state.clone();
                            let vpn_state_change = VpnServiceStateChange::from(new_state);
                            if let Err(e) = self.vpn_state_changes_tx.send(vpn_state_change) {
                                tracing::error!("Failed to send vpn state change: {}", e);
                            }
                        }
                        TunnelEvent::MixnetState(_) => {}
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

        if let Err(e) = self.state_machine_handle.await {
            tracing::error!("Failed to join on state machine handle: {}", e);
        }

        tracing::info!("Exiting vpn service run loop");

        Ok(())
    }

    async fn handle_service_command(&mut self, command: VpnServiceCommand) {
        match command {
            VpnServiceCommand::Info(tx, ()) => {
                let result = self.handle_info().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::SetNetwork(tx, network) => {
                let result = self.handle_set_network(network).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetSystemMessages(tx, ()) => {
                let result = self.handle_get_system_messages().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetFeatureFlags(tx, ()) => {
                let result = self.handle_get_feature_flags().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::Connect(tx, connect_args) => {
                let result = self.handle_connect(connect_args).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::Disconnect(tx, ()) => {
                let result = self.handle_disconnect().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::Status(tx, ()) => {
                let result = self.handle_status().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::StoreAccount(tx, account) => {
                let result = self.handle_store_account(account).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::IsAccountStored(tx, ()) => {
                let result = self.handle_is_account_stored().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::ForgetAccount(tx, ()) => {
                let result = self.handle_forget_account().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetAccountIdentity(tx, ()) => {
                let result = self.handle_get_account_identity().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetAccountLinks(tx, locale) => {
                let result = self.handle_get_account_links(locale).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetAccountState(tx, ()) => {
                let result = self.handle_get_account_state().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::RefreshAccountState(tx, ()) => {
                let result = self.handle_refresh_account_state().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetAccountUsage(tx, ()) => {
                let result = self.handle_get_usage().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::IsReadyToConnect(tx, ()) => {
                let result = Ok(self.handle_is_ready_to_connect().await);
                let _ = tx.send(result);
            }
            VpnServiceCommand::ResetDeviceIdentity(tx, seed) => {
                let result = self.handle_reset_device_identity(seed).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetDeviceIdentity(tx, ()) => {
                let result = self.handle_get_device_identity().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::RegisterDevice(tx, ()) => {
                let result = self.handle_register_device().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetDevices(tx, ()) => {
                let result = self.handle_get_devices().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetActiveDevices(tx, ()) => {
                let result = self.handle_get_active_devices().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::RequestZkNym(tx, ()) => {
                let result = self.handle_request_zk_nym().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetDeviceZkNyms(tx, ()) => {
                let result = self.handle_get_device_zk_nyms().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetZkNymsAvailableForDownload(tx, ()) => {
                let result = self.handle_get_zk_nyms_available_for_download().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetZkNymById(tx, id) => {
                let result = self.handle_get_zk_nym_by_id(id).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::ConfirmZkNymIdDownloaded(tx, id) => {
                let result = self.handle_confirm_zk_nym_id_downloaded(id).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetAvailableTickets(tx, ()) => {
                let result = self.handle_get_available_tickets().await;
                let _ = tx.send(result);
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
            let mut read_config: NymVpnServiceConfig =
                super::config::read_config_file(&self.config_file)
                    .map_err(|err| {
                        tracing::error!(
                            "Failed to read config file, resetting to defaults: {:?}",
                            err
                        );
                    })
                    .unwrap_or_default();
            read_config.entry_point = entry.unwrap_or(read_config.entry_point);
            read_config.exit_point = exit.unwrap_or(read_config.exit_point);
            super::config::write_config_file(&self.config_file, &read_config)
                .map_err(Error::ConfigSetup)?;
            read_config
        } else {
            let config = NymVpnServiceConfig {
                entry_point: entry.unwrap_or(EntryPoint::Random),
                exit_point: exit.unwrap_or(ExitPoint::Random),
            };
            super::config::create_config_file(&self.config_file, config)
                .map_err(Error::ConfigSetup)?
        };
        Ok(config)
    }

    async fn wait_for_ready_to_connect(
        &self,
        credentials_mode: bool,
    ) -> Result<(), AccountCommandError> {
        self.account_command_tx.ensure_update_account().await?;
        self.account_command_tx.ensure_update_device().await?;
        self.account_command_tx.ensure_register_device().await?;
        if credentials_mode {
            self.account_command_tx.ensure_available_zk_nyms().await?;
        }
        Ok(())
    }

    async fn wait_for_ready_to_connect_until_cancelled(
        &self,
        enable_credentials_mode: bool,
    ) -> Result<(), VpnServiceConnectError> {
        let wait_for_ready_to_connect_fut = self
            .wait_for_ready_to_connect(enable_credentials_mode)
            .then(|n| async move {
                n.inspect_err(|err| {
                    tracing::error!("Failed to wait for ready to connect: {:?}", err);
                })
            });
        self.shutdown_token
            .run_until_cancelled(wait_for_ready_to_connect_fut)
            .await
            .ok_or(VpnServiceConnectError::Cancel)?
            .map_err(AccountNotReady::from)?;
        Ok(())
    }

    async fn handle_connect(
        &mut self,
        connect_args: ConnectArgs,
    ) -> Result<(), VpnServiceConnectError> {
        let ConnectArgs {
            entry,
            exit,
            mut options,
        } = connect_args;

        // Get feature flag
        let enable_credentials_mode = self
            .network_env
            .get_feature_flag_credential_mode()
            .unwrap_or(false);
        tracing::debug!("feature flag: credential mode: {enable_credentials_mode}");

        options.enable_credentials_mode =
            options.enable_credentials_mode || enable_credentials_mode;

        // Before attempting to connect, ensure that the account is ready with the account synced,
        // the device registered, and possibly zknym ticketbooks available in local credential
        // storage.
        self.wait_for_ready_to_connect_until_cancelled(options.enable_credentials_mode)
            .await?;

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

        let config = self
            .try_setup_config(entry, exit)
            .map_err(|err| VpnServiceConnectError::Internal(err.to_string()))?;
        tracing::info!("Using config: {}", config);

        let gateway_options = GatewayPerformanceOptions {
            mixnet_min_performance: options
                .min_gateway_mixnet_performance
                .map(|x| x.round_to_integer()),
            vpn_min_performance: options
                .min_gateway_vpn_performance
                .map(|x| x.round_to_integer()),
        };

        tracing::info!(
            "Using statistics recipient: {:?}",
            self.statistics_recipient
        );

        let mixnet_client_config = MixnetClientConfig {
            disable_poisson_rate: options.disable_poisson_rate,
            disable_background_cover_traffic: options.disable_background_cover_traffic,
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
            enable_credentials_mode: options.enable_credentials_mode,
            statistics_recipient: self.statistics_recipient.map(Box::new),
            mixnet_tunnel_options: MixnetTunnelOptions::default(),
            wireguard_tunnel_options: WireguardTunnelOptions {
                multihop_mode: if options.netstack {
                    WireguardMultihopMode::Netstack
                } else {
                    WireguardMultihopMode::TunTun
                },
            },
            gateway_performance_options: gateway_options,
            mixnet_client_config: Some(mixnet_client_config),
            entry_point: Box::new(config.entry_point),
            exit_point: Box::new(config.exit_point),
            dns,
            user_agent: options.user_agent,
        };

        match self
            .command_sender
            .send(TunnelCommand::SetTunnelSettings(tunnel_settings))
        {
            Ok(()) => self
                .command_sender
                .send(TunnelCommand::Connect)
                .map_err(|e| {
                    tracing::error!("Failed to send command to connect: {}", e);
                    VpnServiceConnectError::Internal("failed to send command to connect".to_owned())
                }),
            Err(e) => {
                tracing::error!("Failed to send command to set tunnel options: {}", e);
                Err(VpnServiceConnectError::Internal(
                    "failed to send command to set tunnel options".to_owned(),
                ))
            }
        }
    }

    async fn handle_disconnect(&mut self) -> Result<(), VpnServiceDisconnectError> {
        self.command_sender
            .send(TunnelCommand::Disconnect)
            .map_err(|e| {
                tracing::error!("Failed to send command to disconnect: {}", e);
                VpnServiceDisconnectError::Internal("failed to send dicsonnect command".to_owned())
            })
    }

    async fn handle_status(&self) -> VpnServiceStatus {
        VpnServiceStatus::from(self.tunnel_state.clone())
    }

    async fn handle_info(&self) -> VpnServiceInfo {
        let bin_info = nym_bin_common::bin_info_local_vergen!();

        VpnServiceInfo {
            version: bin_info.build_version.to_string(),
            build_timestamp: time::OffsetDateTime::parse(bin_info.build_timestamp, &Rfc3339).ok(),
            triple: bin_info.cargo_triple.to_string(),
            platform: self.user_agent.platform.clone(),
            git_commit: bin_info.commit_sha.to_string(),
            nym_network: self.network_env.nym_network.clone(),
            nym_vpn_network: self.network_env.nym_vpn_network.clone(),
        }
    }

    async fn handle_set_network(&self, network: String) -> Result<(), SetNetworkError> {
        let mut global_config =
            GlobalConfigFile::read_from_file().map_err(|source| SetNetworkError::ReadConfig {
                source: source.into(),
            })?;

        // Manually restrict the set of possible network, until we handle this automatically
        let network_selected = NetworkEnvironments::try_from(network.as_str())
            .map_err(|_err| SetNetworkError::NetworkNotFound(network.to_owned()))?;
        global_config.network_name = network_selected.to_string();

        global_config
            .write_to_file()
            .map_err(|source| SetNetworkError::WriteConfig {
                source: source.into(),
            })?;

        tracing::info!(
            "Network updated to: {} (SERVICE RESTART REQUIRED!)",
            network_selected
        );

        Ok(())
    }

    async fn handle_get_system_messages(&self) -> SystemMessages {
        self.network_env.nym_vpn_network.system_messages.clone()
    }

    async fn handle_get_feature_flags(&self) -> Option<FeatureFlags> {
        self.network_env.feature_flags.clone()
    }

    async fn handle_store_account(
        &mut self,
        account: Zeroizing<String>,
    ) -> Result<(), AccountError> {
        tracing::info!("Storing account");
        let mnemonic = Mnemonic::parse::<&str>(account.as_ref())?;
        self.account_command_tx
            .store_account(mnemonic)
            .await
            .map_err(|source| AccountError::AccountCommandError { source })
    }

    async fn handle_is_account_stored(&self) -> Result<bool, AccountError> {
        Ok(self.shared_account_state.is_account_stored().await)
    }

    async fn handle_forget_account(&mut self) -> Result<(), AccountError> {
        if self.tunnel_state != TunnelState::Disconnected {
            return Err(AccountError::IsConnected);
        }

        let data_dir = self.data_dir.clone();
        tracing::info!(
            "REMOVING ALL ACCOUNT AND DEVICE DATA IN: {}",
            data_dir.display()
        );

        self.account_command_tx
            .forget_account()
            .await
            .map_err(|source| AccountError::AccountCommandError { source })
    }

    async fn handle_get_account_identity(&self) -> Result<Option<String>, AccountError> {
        Ok(self.shared_account_state.get_account_id().await)
    }

    async fn handle_get_account_links(
        &self,
        locale: String,
    ) -> Result<ParsedAccountLinks, AccountError> {
        let account_id = self.handle_get_account_identity().await?;

        self.network_env
            .nym_vpn_network
            .account_management
            .clone()
            .ok_or(AccountError::AccountManagementNotConfigured)?
            .try_into_parsed_links(&locale, account_id.as_deref())
            .map_err(|err| {
                tracing::error!("Failed to parse account links: {:?}", err);
                AccountError::FailedToParseAccountLinks
            })
    }

    async fn handle_get_account_state(&self) -> Result<AccountStateSummary, AccountError> {
        Ok(self.shared_account_state.lock().await.clone())
    }

    async fn handle_refresh_account_state(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::SyncAccountState(None))
            .map_err(|err| AccountError::AccountControllerError { source: err })
    }

    async fn handle_get_usage(&self) -> Result<Vec<NymVpnUsage>, AccountError> {
        self.account_command_tx
            .get_usage()
            .await
            .map_err(|source| AccountError::AccountCommandError { source })
    }

    async fn handle_is_ready_to_connect(&self) -> ReadyToConnect {
        let credential_mode = false;
        self.shared_account_state
            .is_ready_to_connect(credential_mode)
            .await
    }

    async fn handle_reset_device_identity(
        &mut self,
        seed: Option<[u8; 32]>,
    ) -> Result<(), AccountError> {
        if self.tunnel_state != TunnelState::Disconnected {
            return Err(AccountError::IsConnected);
        }

        // First disconnect the VPN
        self.handle_disconnect()
            .await
            .map_err(|err| AccountError::FailedToResetDeviceKeys {
                source: Box::new(err),
            })?;

        self.storage
            .lock()
            .await
            .reset_keys(seed)
            .await
            .map_err(|err| AccountError::FailedToResetDeviceKeys {
                source: Box::new(err),
            })?;

        self.account_command_tx
            .send(AccountCommand::SyncAccountState(None))
            .map_err(|source| AccountError::AccountControllerError { source })
    }

    async fn handle_get_device_identity(&self) -> Result<String, AccountError> {
        self.account_command_tx
            .get_device_identity()
            .await
            .map_err(|source| AccountError::AccountCommandError { source })
    }

    async fn handle_register_device(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::RegisterDevice(None))
            .map_err(|source| AccountError::AccountControllerError { source })
    }

    async fn handle_get_devices(&self) -> Result<Vec<NymVpnDevice>, AccountError> {
        self.account_command_tx
            .get_devices()
            .await
            .map_err(|source| AccountError::AccountCommandError { source })
    }

    async fn handle_get_active_devices(&self) -> Result<Vec<NymVpnDevice>, AccountError> {
        self.account_command_tx
            .get_active_devices()
            .await
            .map_err(|source| AccountError::AccountCommandError { source })
    }

    async fn handle_request_zk_nym(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::RequestZkNym(None))
            .map_err(|source| AccountError::AccountControllerError { source })
    }

    async fn handle_get_device_zk_nyms(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::GetDeviceZkNym)
            .map_err(|source| AccountError::AccountControllerError { source })
    }

    async fn handle_get_zk_nyms_available_for_download(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::GetZkNymsAvailableForDownload)
            .map_err(|source| AccountError::AccountControllerError { source })
    }

    async fn handle_get_zk_nym_by_id(&self, id: String) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::GetZkNymById(id))
            .map_err(|source| AccountError::AccountControllerError { source })
    }

    async fn handle_confirm_zk_nym_id_downloaded(&self, id: String) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::ConfirmZkNymIdDownloaded(id))
            .map_err(|source| AccountError::AccountControllerError { source })
    }

    async fn handle_get_available_tickets(&self) -> Result<AvailableTicketbooks, AccountError> {
        self.account_command_tx
            .get_available_tickets()
            .await
            .map_err(|source| AccountError::AccountCommandError { source })
    }
}
