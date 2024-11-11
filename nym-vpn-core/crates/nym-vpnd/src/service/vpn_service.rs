// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use bip39::Mnemonic;
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
    AccountCommand, AccountController, AccountStateSummary, AvailableTicketbooks, ReadyToConnect,
    SharedAccountState,
};
use nym_vpn_api_client::{
    response::{NymVpnAccountSummaryResponse, NymVpnDevicesResponse},
    types::{Percent, VpnApiAccount},
};
use nym_vpn_lib::{
    gateway_directory::{self, EntryPoint, ExitPoint},
    tunnel_state_machine::{
        ConnectionData, DnsOptions, GatewayPerformanceOptions, MixnetEvent, MixnetTunnelOptions,
        NymConfig, TunnelCommand, TunnelConnectionData, TunnelEvent, TunnelSettings, TunnelState,
        TunnelStateMachine, TunnelType,
    },
    MixnetClientConfig, NodeIdentity, Recipient,
};

use crate::config::GlobalConfigFile;

use super::{
    config::{ConfigSetupError, NetworkEnvironments, NymVpnServiceConfig, DEFAULT_CONFIG_FILE},
    error::{AccountError, AccountNotReady, ConnectionFailedError, Error, Result, SetNetworkError},
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
pub enum VpnServiceCommand {
    Info(oneshot::Sender<VpnServiceInfo>, ()),
    SetNetwork(oneshot::Sender<Result<(), SetNetworkError>>, String),
    GetSystemMessages(oneshot::Sender<SystemMessages>, ()),
    GetFeatureFlags(oneshot::Sender<Option<FeatureFlags>>, ()),
    Connect(
        oneshot::Sender<Result<(), VpnServiceConnectError>>,
        (ConnectArgs, nym_vpn_lib::UserAgent),
    ),
    Disconnect(oneshot::Sender<Result<(), VpnServiceDisconnectError>>, ()),
    Status(oneshot::Sender<VpnServiceStatus>, ()),
    StoreAccount(oneshot::Sender<Result<(), AccountError>>, String),
    IsAccountStored(oneshot::Sender<Result<bool, AccountError>>, ()),
    RemoveAccount(oneshot::Sender<Result<(), AccountError>>, ()),
    GetAccountIdentity(oneshot::Sender<Result<String, AccountError>>, ()),
    GetAccountLinks(
        oneshot::Sender<Result<ParsedAccountLinks, AccountError>>,
        Locale,
    ),
    GetAccountState(
        oneshot::Sender<Result<AccountStateSummary, AccountError>>,
        (),
    ),
    RefreshAccountState(oneshot::Sender<Result<(), AccountError>>, ()),
    IsReadyToConnect(oneshot::Sender<Result<ReadyToConnect, AccountError>>, ()),
    ResetDeviceIdentity(oneshot::Sender<Result<(), AccountError>>, Option<Seed>),
    GetDeviceIdentity(oneshot::Sender<Result<String, AccountError>>, ()),
    RegisterDevice(oneshot::Sender<Result<(), AccountError>>, ()),
    RequestZkNym(oneshot::Sender<Result<(), AccountError>>, ()),
    GetDeviceZkNyms(oneshot::Sender<Result<(), AccountError>>, ()),
    GetZkNymsAvailableForDownload(oneshot::Sender<Result<(), AccountError>>, ()),
    GetZkNymById(oneshot::Sender<Result<(), AccountError>>, String),
    ConfirmZkNymIdDownloaded(oneshot::Sender<Result<(), AccountError>>, String),
    GetAvailableTickets(
        oneshot::Sender<Result<AvailableTicketbooks, AccountError>>,
        (),
    ),
    FetchRawAccountSummary(
        oneshot::Sender<Result<NymVpnAccountSummaryResponse, AccountError>>,
        (),
    ),
    FetchRawDevices(
        oneshot::Sender<Result<NymVpnDevicesResponse, AccountError>>,
        (),
    ),
}

impl fmt::Display for VpnServiceCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnServiceCommand::Info(..) => write!(f, "Info"),
            VpnServiceCommand::SetNetwork(..) => write!(f, "SetNetwork"),
            VpnServiceCommand::GetSystemMessages(..) => write!(f, "GetSystemMessages"),
            VpnServiceCommand::GetFeatureFlags(..) => write!(f, "GetFeatureFlags"),
            VpnServiceCommand::Connect(_, (args, user_agent)) => {
                write!(f, "Connect {{ {args:?}, {user_agent:?} }}")
            }
            VpnServiceCommand::Disconnect(..) => write!(f, "Disconnect"),
            VpnServiceCommand::Status(..) => write!(f, "Status"),
            VpnServiceCommand::StoreAccount(..) => write!(f, "StoreAccount"),
            VpnServiceCommand::IsAccountStored(..) => write!(f, "IsAccountStored"),
            VpnServiceCommand::RemoveAccount(..) => write!(f, "RemoveAccount"),
            VpnServiceCommand::GetAccountIdentity(..) => write!(f, "GetAccountIdentity"),
            VpnServiceCommand::GetAccountLinks(..) => write!(f, "GetAccountLinks"),
            VpnServiceCommand::GetAccountState(..) => write!(f, "GetAccountState"),
            VpnServiceCommand::RefreshAccountState(..) => write!(f, "RefreshAccountState"),
            VpnServiceCommand::IsReadyToConnect(..) => write!(f, "IsReadyToConnect"),
            VpnServiceCommand::ResetDeviceIdentity(..) => write!(f, "ResetDeviceIdentity"),
            VpnServiceCommand::GetDeviceIdentity(..) => write!(f, "GetDeviceIdentity"),
            VpnServiceCommand::RegisterDevice(..) => write!(f, "RegisterDevice"),
            VpnServiceCommand::RequestZkNym(..) => write!(f, "RequestZkNym"),
            VpnServiceCommand::GetDeviceZkNyms(..) => write!(f, "GetDeviceZkNyms"),
            VpnServiceCommand::GetZkNymsAvailableForDownload(..) => {
                write!(f, "GetZkNymsAvailableForDownload")
            }
            VpnServiceCommand::GetZkNymById(..) => write!(f, "GetZkNymById"),
            VpnServiceCommand::ConfirmZkNymIdDownloaded(..) => {
                write!(f, "ConfirmZkNymIdDownloaded")
            }
            VpnServiceCommand::GetAvailableTickets(..) => write!(f, "GetAvailableTickets"),
            VpnServiceCommand::FetchRawAccountSummary(..) => write!(f, "FetchRawAccountSummery"),
            VpnServiceCommand::FetchRawDevices(..) => write!(f, "FetchRawDevices"),
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
    pub(crate) disable_poisson_rate: bool,
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
            TunnelState::Disconnected => Self::NotConnected,
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
    // The network environment
    network_env: Network,

    // The account state, updated by the account controller
    shared_account_state: SharedAccountState,

    // Listen for commands from the command interface, like the grpc listener that listens user
    // commands.
    vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,

    vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
    status_tx: broadcast::Sender<MixnetEvent>,

    // Send commands to the account controller
    account_command_tx: mpsc::UnboundedSender<AccountCommand>,

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
        network_env: Network,
    ) -> JoinHandle<()> {
        tracing::info!("Starting VPN service");
        tokio::spawn(async {
            match NymVpnService::new(
                vpn_state_changes_tx,
                vpn_command_rx,
                status_tx,
                shutdown_token,
                network_env,
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
        network_env: Network,
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

        // We need to create the user agent here and not in the controller so that we correctly
        // pick up build time constants.
        let user_agent = crate::util::construct_user_agent();
        let account_controller = AccountController::new(
            Arc::clone(&storage),
            data_dir.clone(),
            user_agent,
            shutdown_token.child_token(),
        )
        .await
        .map_err(|source| Error::Account(AccountError::AccountControllerError { source }))?;

        let shared_account_state = account_controller.shared_state();
        let account_command_tx = account_controller.command_tx();
        let _account_controller_handle = tokio::task::spawn(account_controller.run());

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
        loop {
            tokio::select! {
                Some(command) = self.vpn_command_rx.recv() => {
                    tracing::debug!("VPN: Received command: {command}");
                    self.handle_service_command(command).await;
                }
                Some(event) = self.event_receiver.recv() => {
                    tracing::info!("Tunnel event: {}", event);
                    match event {
                        TunnelEvent::NewState(new_state) => {
                            self.tunnel_state = new_state.clone();
                            let vpn_state_change = VpnServiceStateChange::from(new_state);
                            if let Err(e) = self.vpn_state_changes_tx.send(vpn_state_change) {
                                tracing::error!("Failed to send vpn state change: {}", e);
                            }
                        }
                        TunnelEvent::MixnetState(event) => {
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
            VpnServiceCommand::Connect(tx, (connect_args, user_agent)) => {
                let result = self.handle_connect(connect_args, user_agent).await;
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
            VpnServiceCommand::RemoveAccount(tx, ()) => {
                let result = self.handle_remove_account().await;
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
            VpnServiceCommand::FetchRawAccountSummary(tx, ()) => {
                let result = self.handle_fetch_raw_account_summary().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::FetchRawDevices(tx, ()) => {
                let result = self.handle_fetch_raw_devices().await;
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

    async fn wait_for_ready_to_connect(&self) -> Result<(), VpnServiceConnectError> {
        match self
            .shared_account_state
            .wait_for_ready_to_connect(Duration::from_secs(10))
            .await
        {
            Some(is_ready) => match is_ready {
                ReadyToConnect::Ready => Ok(()),
                not_ready => {
                    tracing::info!("Not ready to connect: {:?}", not_ready);
                    Err(VpnServiceConnectError::Account(
                        AccountNotReady::try_from(not_ready)
                            .map_err(|err| VpnServiceConnectError::Internal(err.to_string()))?,
                    ))
                }
            },
            None => Err(VpnServiceConnectError::Internal("timeout".to_owned())),
        }
    }

    async fn handle_connect(
        &mut self,
        connect_args: ConnectArgs,
        _user_agent: nym_vpn_lib::UserAgent, // todo: use user-agent!
    ) -> Result<(), VpnServiceConnectError> {
        let wait_for_ready_to_connect_fut = self.wait_for_ready_to_connect();
        self.shutdown_token
            .run_until_cancelled(wait_for_ready_to_connect_fut)
            .await
            .ok_or(VpnServiceConnectError::Cancel)??;

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
                .map_err(|e| {
                    tracing::error!("Failed to send command to connect: {}", e);
                    VpnServiceConnectError::Internal("failed to send tunnel command".to_owned())
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
        let user_agent = crate::util::construct_user_agent();

        VpnServiceInfo {
            version: bin_info.build_version.to_string(),
            build_timestamp: time::OffsetDateTime::parse(bin_info.build_timestamp, &Rfc3339).ok(),
            triple: bin_info.cargo_triple.to_string(),
            platform: user_agent.platform,
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
            .send(AccountCommand::UpdateAccountState)
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
            .send(AccountCommand::UpdateAccountState)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })?;

        Ok(())
    }

    async fn handle_get_account_identity(&self) -> Result<String, AccountError> {
        self.load_account().await.map(|account| account.id())
    }

    async fn handle_get_account_links(
        &self,
        locale: String,
    ) -> Result<ParsedAccountLinks, AccountError> {
        let account = self.load_account().await?;
        let account_id = account.id();

        self.network_env
            .nym_vpn_network
            .account_management
            .clone()
            .ok_or(AccountError::AccountManagementNotConfigured)?
            .try_into_parsed_links(&locale, &account_id)
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
            .send(AccountCommand::UpdateAccountState)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })
    }

    async fn handle_is_ready_to_connect(&self) -> ReadyToConnect {
        self.shared_account_state.is_ready_to_connect().await
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

    async fn handle_reset_device_identity(
        &self,
        seed: Option<[u8; 32]>,
    ) -> Result<(), AccountError> {
        self.storage
            .lock()
            .await
            .reset_keys(seed)
            .await
            .map_err(|err| AccountError::FailedToResetKeys {
                source: Box::new(err),
            })?;

        self.account_command_tx
            .send(AccountCommand::UpdateAccountState)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })?;

        Ok(())
    }

    async fn handle_get_device_identity(&self) -> Result<String, AccountError> {
        self.load_device_keys()
            .await
            .map(|keys| keys.device_keypair().public_key().to_string())
    }

    async fn handle_register_device(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::RegisterDevice)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })
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

    async fn handle_get_zk_nyms_available_for_download(&self) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::GetZkNymsAvailableForDownload)
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })
    }

    async fn handle_get_zk_nym_by_id(&self, id: String) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::GetZkNymById(id))
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })
    }

    async fn handle_confirm_zk_nym_id_downloaded(&self, id: String) -> Result<(), AccountError> {
        self.account_command_tx
            .send(AccountCommand::ConfirmZkNymIdDownloaded(id))
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })
    }

    async fn handle_get_available_tickets(&self) -> Result<AvailableTicketbooks, AccountError> {
        let (result_tx, result_rx) = oneshot::channel();
        self.account_command_tx
            .send(AccountCommand::GetAvailableTickets(result_tx))
            .map_err(|err| AccountError::SendCommand {
                source: Box::new(err),
            })?;
        let result = result_rx.await.map_err(|err| AccountError::RecvCommand {
            source: Box::new(err),
        })?;
        result.map_err(|err| AccountError::AccountControllerError { source: err })
    }

    async fn handle_fetch_raw_account_summary(
        &self,
    ) -> Result<NymVpnAccountSummaryResponse, AccountError> {
        if !self.handle_is_account_stored().await? {
            return Err(AccountError::NoAccountStored);
        }

        // Get account
        let account = self.load_account().await?;

        // Setup client
        let nym_vpn_api_url = self.network_env.vpn_api_url();
        let user_agent = crate::util::construct_user_agent();
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client
            .get_account_summary(&account)
            .await
            .map_err(Into::into)
    }

    async fn handle_fetch_raw_devices(&self) -> Result<NymVpnDevicesResponse, AccountError> {
        if !self.handle_is_account_stored().await? {
            return Err(AccountError::NoAccountStored);
        }

        // Get account
        let account = self.load_account().await?;

        // Setup client
        let nym_vpn_api_url = self.network_env.vpn_api_url();
        let user_agent = crate::util::construct_user_agent();
        let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent)?;

        api_client.get_devices(&account).await.map_err(Into::into)
    }
}
