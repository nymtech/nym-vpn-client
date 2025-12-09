// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{FutureExt, StreamExt, future::Fuse, pin_mut};
use std::{net::IpAddr, path::PathBuf, pin::Pin, sync::Arc};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::{
    sync::{RwLock, broadcast, mpsc, oneshot, watch},
    task::JoinHandle,
    time::{Duration, Instant},
};
use tokio_stream::wrappers::WatchStream;
use tokio_util::sync::CancellationToken;

use super::{
    Socks5Error, Socks5Service, Socks5Status,
    config::{NetworkEnvironments, VpnServiceConfigManager},
    error::{
        AccountControllerError, AccountLinksError, Error, GlobalConfigError, ListGatewaysError,
        Result, SetNetworkError,
    },
    socks5_idle_timeout, socks5_request_timeout,
};
use crate::{config::GlobalConfig, logging::LogFileRemoverHandle};
use bip39::Mnemonic;
use futures::{FutureExt, StreamExt, future::Fuse, pin_mut};
use nym_common::trace_err_chain;
use nym_statistics::{
    StatisticsCommandsSender, StatisticsController, StatisticsControllerError, StatisticsSender,
};
use nym_vpn_account_controller::{
    AccountCommandSender, AccountController, AccountControllerConfig, AccountStateReceiver,
    AvailableTicketbooks, NyxdClient,
};
use nym_vpn_api_client::api_urls_to_urls;
use nym_vpn_lib::{
    DEFAULT_DNS_SERVERS, NodeIdentity, UserAgent, VpnTopologyProvider,
    gateway_directory::{self, GatewayCache, GatewayCacheHandle, GatewayClient},
    tunnel_state_machine::{NymConfig, TunnelCommand, TunnelConstants, TunnelStateMachine},
};
use nym_vpn_lib_types::{
    AccountBalanceResponse, AccountCommandError, AccountControllerState,
    DecentralisedObtainTicketbooksRequest, EnableSocks5Request, EntryPoint, ExitPoint,
    FeatureFlags, Gateway, GatewayFilters, ListGatewaysOptions, LogPath, NetworkCompatibility,
    NetworkStatisticsIdentity, NymNetworkDetails, NymVpnDevice, NymVpnNetwork, NymVpnUsage,
    ParsedAccountLinks, StoreAccountRequest, SystemMessage, TargetState, TunnelEvent, TunnelState,
    VpnServiceConfig, VpnServiceInfo,
};
use nym_vpn_network_config::{DiscoveryRefresher, DiscoveryRefresherEvent, Network};
use nym_vpn_store::types::{StorableAccount, StoredAccountMode};
use std::{net::IpAddr, path::PathBuf, pin::Pin, sync::Arc};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::{
    sync::{RwLock, broadcast, mpsc, oneshot, watch},
    task::JoinHandle,
    time::{Duration, Instant},
};
use tokio_stream::wrappers::WatchStream;
use tokio_util::sync::CancellationToken;

// Seed used to generate device identity keys
type Seed = [u8; 32];

type Locale = String;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, strum::Display)]
pub enum VpnServiceCommand {
    Info(oneshot::Sender<VpnServiceInfo>, ()),
    GetConfig(oneshot::Sender<VpnServiceConfig>, ()),
    SetEntryPoint(oneshot::Sender<()>, EntryPoint),
    SetExitPoint(oneshot::Sender<()>, ExitPoint),
    SetDisableIPv6(oneshot::Sender<()>, bool),
    SetEnableTwoHop(oneshot::Sender<()>, bool),
    SetNetstack(oneshot::Sender<()>, bool),
    SetAllowLan(oneshot::Sender<()>, bool),
    SetEnableBridges(oneshot::Sender<()>, bool),
    SetResidentialExit(oneshot::Sender<()>, bool),
    SetEnableCustomDns(oneshot::Sender<()>, bool),
    SetCustomDns(oneshot::Sender<()>, Vec<IpAddr>),
    SetNetwork(oneshot::Sender<Result<(), SetNetworkError>>, String),
    GetSystemMessages(oneshot::Sender<Vec<SystemMessage>>, ()),
    GetNetworkCompatibility(oneshot::Sender<Option<NetworkCompatibility>>, ()),
    GetFeatureFlags(oneshot::Sender<Option<FeatureFlags>>, ()),
    GetDefaultDns(oneshot::Sender<Vec<IpAddr>>, ()),
    ListGateways(
        oneshot::Sender<Result<Vec<Gateway>, ListGatewaysError>>,
        ListGatewaysOptions,
    ),
    ListFilteredGateways(
        oneshot::Sender<Result<Vec<Gateway>, ListGatewaysError>>,
        GatewayFilters,
    ),
    EnableSocks5(
        oneshot::Sender<Result<(), Socks5Error>>,
        EnableSocks5Request,
    ),
    DisableSocks5(oneshot::Sender<Result<(), Socks5Error>>, ()),
    GetSocks5Status(oneshot::Sender<Result<Socks5Status, Socks5Error>>, ()),
    SetTargetState(oneshot::Sender<bool>, TargetState),
    Reconnect(oneshot::Sender<bool>, ()),
    GetTunnelState(oneshot::Sender<TunnelState>, ()),
    StoreAccount(
        oneshot::Sender<Result<(), AccountCommandError>>,
        StoreAccountRequest,
    ),
    DecentralisedBalance(oneshot::Sender<AccountBalanceResponse>, ()),
    DecentralisedObtainTicketbooks(
        oneshot::Sender<Result<(), AccountCommandError>>,
        DecentralisedObtainTicketbooksRequest,
    ),
    IsAccountStored(oneshot::Sender<bool>, ()),
    ForgetAccount(oneshot::Sender<Result<(), AccountCommandError>>, ()),
    RotateKeys(oneshot::Sender<Result<(), AccountCommandError>>, ()),
    GetAccountIdentity(
        oneshot::Sender<Result<Option<String>, AccountCommandError>>,
        (),
    ),
    GetAccountLinks(
        oneshot::Sender<Result<ParsedAccountLinks, AccountLinksError>>,
        Locale,
    ),
    GetAccountState(oneshot::Sender<AccountControllerState>, ()),
    RefreshAccountState(oneshot::Sender<()>, ()),
    GetAccountUsage(
        oneshot::Sender<Result<Vec<NymVpnUsage>, AccountCommandError>>,
        (),
    ),
    ResetDeviceIdentity(
        oneshot::Sender<Result<(), AccountCommandError>>,
        Option<Seed>,
    ),
    GetDeviceIdentity(
        oneshot::Sender<Result<Option<String>, AccountCommandError>>,
        (),
    ),
    GetDevices(
        oneshot::Sender<Result<Vec<NymVpnDevice>, AccountCommandError>>,
        (),
    ),
    GetActiveDevices(
        oneshot::Sender<Result<Vec<NymVpnDevice>, AccountCommandError>>,
        (),
    ),
    GetAvailableTickets(
        oneshot::Sender<Result<AvailableTicketbooks, AccountCommandError>>,
        (),
    ),
    GetLogPath(oneshot::Sender<Option<LogPath>>, ()),
    DeleteLogFile(oneshot::Sender<()>, ()),
    IsSentryEnabled(oneshot::Sender<bool>, ()),
    ToggleSentry(oneshot::Sender<Result<(), GlobalConfigError>>, bool),
    AllowDisconnectedNetStats(oneshot::Sender<()>, bool),
    EnableNetStats(oneshot::Sender<()>, bool),
    ResetNetStatsSeed(
        oneshot::Sender<Result<(), StatisticsControllerError>>,
        Option<String>,
    ),
    GetNetStatsSeed(
        oneshot::Sender<Result<NetworkStatisticsIdentity, StatisticsControllerError>>,
        (),
    ),
}

pub struct NymVpnServiceParameters {
    pub log_path: Option<LogPath>,
    pub network_env: Box<Network>,
    pub sentry_enabled: bool,
    pub user_agent: UserAgent,
}

pub struct NymVpnService {
    // The network environment
    network_tx: watch::Sender<Box<Network>>,

    // The user agent used for HTTP request
    user_agent: UserAgent,

    // Listen for commands from the command interface, like the grpc listener that listens user
    // commands.
    vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,

    // Send command to delete and recreate logging file
    log_file_remover_handle: Option<LogFileRemoverHandle>,

    // Send commands to the account controller
    account_command_tx: AccountCommandSender,

    // Receive state from account controller,
    account_state_rx: AccountStateReceiver,

    // Path to the data directory
    data_dir: PathBuf,

    // If log to file is enabled, path to the log directory and log filename
    log_path: Option<LogPath>,

    // Broadcast channel for sending tunnel events to the outside world
    tunnel_event_tx: broadcast::Sender<TunnelEvent>,

    // Target state
    target_state: TargetState,

    // Last known tunnel state
    tunnel_state: Arc<RwLock<TunnelState>>,

    // Timer used to throttle changes to tunnel settings
    tunnel_settings_update_timer: Pin<Box<Fuse<tokio::time::Sleep>>>,

    // Command channel for state machine
    command_sender: mpsc::UnboundedSender<TunnelCommand>,

    // Event channel for receiving events from state machine
    event_receiver: mpsc::UnboundedReceiver<TunnelEvent>,

    // Tunnel state machine handle
    state_machine_handle: Option<JoinHandle<()>>,

    // Account controller handle
    account_controller_handle: JoinHandle<()>,

    // Statistics controller handle
    statistics_controller_handle: JoinHandle<()>,

    // Configuration Manager
    config_manager: VpnServiceConfigManager,

    // Gateway cache join handle
    gateway_cache_join_handle: JoinHandle<()>,

    // Gateway cache handle
    gateway_cache_handle: GatewayCacheHandle,

    // Discovery refresher event receiver
    discovery_refresher_event_rx: mpsc::UnboundedReceiver<DiscoveryRefresherEvent>,

    // Discovery refresher join handle
    discovery_refresher_join_handle: JoinHandle<()>,

    // VPN service shutdown token.
    shutdown_token: CancellationToken,

    // Shutdown token used by state machine
    state_machine_shutdown_token: CancellationToken,

    // Shutdown token used for account and statistics controllers and other services that are safe to exit altogether.
    services_shutdown_token: CancellationToken,

    // Sentry client has been initialized and is enabled
    sentry_enabled: bool,

    // The statistics channel sender
    statistics_event_sender: StatisticsSender,

    // The stats control command channel,
    stats_control_commands_sender: StatisticsCommandsSender,

    // Lazy SOCKS5 proxy service handle
    socks5_service: Socks5Service,
}

impl NymVpnService {
    pub fn spawn(
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        tunnel_event_tx: broadcast::Sender<TunnelEvent>,
        log_file_remover_handle: Option<LogFileRemoverHandle>,
        parameters: NymVpnServiceParameters,
        shutdown_token: CancellationToken,
    ) -> JoinHandle<()> {
        tracing::trace!("Starting VPN service");
        tokio::spawn(async move {
            let Ok(service) = Box::pin(NymVpnService::new(
                vpn_command_rx,
                tunnel_event_tx,
                log_file_remover_handle,
                parameters,
                shutdown_token,
            ))
            .await
            .inspect_err(|err| {
                trace_err_chain!(err, "Failed to initialize VPN service");
            }) else {
                return;
            };

            tracing::debug!("VPN service initialized successfully");

            match service.run().await {
                Ok(_) => {
                    tracing::info!("VPN service has successfully exited");
                }
                Err(e) => {
                    tracing::error!("VPN service has exited with error: {e:?}");
                }
            }
        })
    }

    pub async fn new(
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        tunnel_event_tx: broadcast::Sender<TunnelEvent>,
        log_file_remover_handle: Option<LogFileRemoverHandle>,
        parameters: NymVpnServiceParameters,
        shutdown_token: CancellationToken,
    ) -> Result<Self> {
        let network_name = parameters
            .network_env
            .nym_network_details()
            .network_name
            .clone();

        let config_dir = super::config::config_dir().join(&network_name);
        let data_dir = super::config::data_dir();
        let network_data_dir = data_dir.join(&network_name);

        let storage = nym_vpn_lib::storage::VpnClientOnDiskStorage::new(network_data_dir.clone());

        // Make sure the data dir exists
        super::config::create_data_dir(&data_dir, &network_name)
            .await
            .map_err(Error::ConfigSetup)?;

        let state_machine_shutdown_token = CancellationToken::new();
        let services_shutdown_token = CancellationToken::new();

        #[cfg(target_os = "linux")]
        let routing_params = nym_vpn_lib::tunnel_state_machine::RoutingParameters::default();

        let route_handler = nym_vpn_lib::tunnel_state_machine::RouteHandler::new(
            #[cfg(target_os = "linux")]
            routing_params,
        )
        .await
        .map_err(nym_vpn_lib::tunnel_state_machine::Error::CreateRouteHandler)
        .map_err(Error::StateMachine)?;

        let tunnel_constants = TunnelConstants::default();
        let connectivity_handle = nym_offline_monitor::spawn_monitor(
            route_handler.inner_handle(),
            #[cfg(target_os = "linux")]
            Some(tunnel_constants.fwmark),
        )
        .await;

        let account_controller_config = AccountControllerConfig {
            data_dir: network_data_dir.clone(),
            credentials_mode: None,
            network_env: *parameters.network_env.clone(),
        };

        let nym_vpn_api_client = nym_vpn_api_client::VpnApiClient::from_network(
            parameters.network_env.nym_network_details(),
            parameters.user_agent.clone(),
            None,
        )
        .await
        .map_err(|err| {
            trace_err_chain!(err, "Failed to create NymVPN API client");
            AccountControllerError::Initialization {
                reason: err.to_string(),
            }
        })?;

        let nyxd_client = NyxdClient::new(&parameters.network_env);

        let account_controller = AccountController::new(
            nym_vpn_api_client,
            nyxd_client,
            account_controller_config,
            storage,
            connectivity_handle.clone(),
            services_shutdown_token.child_token(),
        )
        .await
        .map_err(|err| {
            tracing::error!("Failed to create account controller: {err:?}");
            AccountControllerError::Initialization {
                reason: err.to_string(),
            }
        })?;

        // These are used to interact with the account controller
        let account_command_tx = account_controller.get_command_sender();
        let account_state_rx = account_controller.get_state_receiver();
        let wireguard_keys_db = account_controller.get_wireguard_keys_storage();
        let account_controller_handle = tokio::task::spawn(account_controller.run());

        let config_manager =
            VpnServiceConfigManager::new(&config_dir, Some(tunnel_event_tx.clone())).await?;

        // Statistics collection setup
        let statistics_controller_config = config_manager.config().network_stats;

        let statistics_api_url = parameters
            .network_env
            .system_configuration
            .as_ref()
            .and_then(|config| config.statistics_api.clone());

        let stats_api_client = statistics_api_url.and_then(|url| nym_statistics_api_client::StatisticsApiClient::new(url.clone(), parameters.user_agent.clone()).inspect_err(|e| tracing::error!("Failed to build Statistics API client. Statistics collection will be disabled : {e}")).ok());

        // Statistics collection can technically fail, but if it's the case, we just disable it as it is not operation critical.
        let statistics_controller = StatisticsController::new(
            statistics_controller_config,
            stats_api_client,
            network_data_dir.clone(),
            services_shutdown_token.child_token(),
        )
        .await;

        let statistics_event_sender = statistics_controller.get_statistics_sender();
        let stats_control_commands_sender = statistics_controller.get_commands_sender();
        let statistics_controller_handle = tokio::task::spawn(statistics_controller.run());

        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));

        // Initialize lazy SOCKS5 service (disabled by default)
        let socks5_service = Socks5Service::new(tunnel_state.clone());

        // These used to interact with the tunnel state machine
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let tunnel_settings = config_manager.generate_tunnel_settings();
        let nyxd_url = parameters.network_env.nyxd_url();

        let Some(nym_api_urls) = parameters.network_env.nym_api_urls() else {
            return Err(Error::AccountController(
                AccountControllerError::Initialization {
                    reason: "Nym API URLs are empty".to_string(),
                },
            ));
        };

        let Some(nym_vpn_api_urls) = parameters.network_env.nym_vpn_api_urls() else {
            return Err(Error::AccountController(
                AccountControllerError::Initialization {
                    reason: "Nym VPN API URLs are empty".to_string(),
                },
            ));
        };

        let gateway_config =
            gateway_directory::Config::new(nyxd_url, nym_api_urls.clone(), nym_vpn_api_urls, None)
                .map_err(|e| AccountControllerError::Initialization {
                    reason: e.to_string(),
                })?;

        let (network_tx, network_rx) = watch::channel(parameters.network_env.clone());
        let nym_config = NymConfig {
            config_path: Some(config_dir.clone()),
            data_path: Some(network_data_dir.clone()),
            gateway_config: gateway_config.clone(),
            network_rx,
        };

        let gateway_directory_client =
            GatewayClient::new(gateway_config, parameters.user_agent.clone())
                .await
                .map_err(|err| {
                    tracing::error!("Failed to create gateway directory client: {err:?}");
                    AccountControllerError::Initialization {
                        reason: err.to_string(),
                    }
                })?;
        let (gateway_cache_handle, gateway_cache_join_handle) = GatewayCache::spawn(
            gateway_directory_client.clone(),
            connectivity_handle.clone(),
            services_shutdown_token.child_token(),
        );

        let urls = api_urls_to_urls(&nym_api_urls).map_err(|err| {
            tracing::error!("Failed to convert Nym API URLs: {err:?}");
            AccountControllerError::Initialization {
                reason: err.to_string(),
            }
        })?;

        let topology_provider = VpnTopologyProvider::new(
            urls,
            parameters.user_agent.clone(),
            false,
            services_shutdown_token.child_token(),
        )
        .await?;
        topology_provider.fetch().await;

        let (discovery_refresher_event_tx, discovery_refresher_event_rx) =
            mpsc::unbounded_channel();
        let (discovery_refresher_command_tx, discovery_refresher_command_rx) =
            mpsc::unbounded_channel();
        let discovery_refresher_join_handle = DiscoveryRefresher::spawn(
            config_dir.clone(),
            parameters.network_env,
            discovery_refresher_command_rx,
            discovery_refresher_event_tx,
            connectivity_handle.clone(),
            services_shutdown_token.child_token(),
        )
        .await
        .map_err(|err| {
            tracing::error!("Failed to start Discovery Refresher: {err:?}");
            AccountControllerError::Initialization {
                reason: err.to_string(),
            }
        })?;

        let state_machine_handle = TunnelStateMachine::spawn(
            command_receiver,
            event_sender,
            nym_config,
            tunnel_settings,
            tunnel_constants,
            account_command_tx.clone(),
            account_state_rx.clone(),
            statistics_event_sender.clone(),
            gateway_cache_handle.clone(),
            topology_provider,
            connectivity_handle,
            discovery_refresher_command_tx,
            wireguard_keys_db,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_handler,
            parameters.user_agent.clone(),
            state_machine_shutdown_token.child_token(),
        )
        .await
        .map_err(Error::StateMachine)?;

        Ok(Self {
            network_tx,
            user_agent: parameters.user_agent,
            vpn_command_rx,
            tunnel_event_tx,
            log_file_remover_handle,
            account_command_tx,
            account_state_rx,
            data_dir: network_data_dir,
            log_path: parameters.log_path,
            target_state: TargetState::Unsecured,
            tunnel_state,
            tunnel_settings_update_timer: Box::pin(Fuse::terminated()),
            state_machine_handle: Some(state_machine_handle),
            account_controller_handle,
            statistics_controller_handle,
            config_manager,
            command_sender,
            event_receiver,
            shutdown_token,
            services_shutdown_token,
            state_machine_shutdown_token,
            gateway_cache_handle,
            gateway_cache_join_handle,
            discovery_refresher_event_rx,
            discovery_refresher_join_handle,
            sentry_enabled: parameters.sentry_enabled,
            statistics_event_sender,
            stats_control_commands_sender,
            socks5_service,
        })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        // Skip the initial account state value
        let mut account_state_rx = WatchStream::new(self.account_state_rx.subscribe()).skip(1);

        loop {
            tokio::select! {
                Some(command) = self.vpn_command_rx.recv() => {
                    self.handle_service_command_timed(command).await;
                }
                Some(event) = self.event_receiver.recv() => {
                    self.handle_tunnel_event(event);
                }
                Some(account_state) = account_state_rx.next() => {
                    self.handle_account_state_change(account_state);
                }
                Some(event) = self.discovery_refresher_event_rx.recv() => {
                    self.handle_discovery_refresher_event(event);
                }
                _ = &mut self.tunnel_settings_update_timer => {
                    self.update_tunnel_settings();
                }
                _ = self.shutdown_token.cancelled() => {
                    tracing::info!("Received shutdown signal");
                    break;
                }
            }
        }

        // Cancel state machine first and wait for it to complete
        self.state_machine_shutdown_token.cancel();

        if let Some(state_machine_handle) = self.state_machine_handle.take() {
            // Drain tunnel events channel and wait for the tunnel state machine to quit
            let fused_state_machine_handle = state_machine_handle.fuse();
            pin_mut!(fused_state_machine_handle);

            loop {
                tokio::select! {
                    result = &mut fused_state_machine_handle => {
                        if let Err(e) = result {
                            tracing::error!("Failed to join on state machine handle: {}", e);
                        }
                        // The loop will continue until `event_receiver` is fully drained
                        self.event_receiver.close();
                    }
                    event = self.event_receiver.recv() => {
                        match event {
                            Some(event) => self.handle_tunnel_event(event),
                            None => break,
                        }
                    }
                }
            }
        }

        // Cancel all other services and wait for them to complete
        self.services_shutdown_token.cancel();

        // Shutdown SOCKS5 service
        self.socks5_service.shutdown().await;

        if let Err(e) = self.account_controller_handle.await {
            tracing::error!("Failed to join on account controller handle: {e}");
        }

        if let Err(e) = self.statistics_controller_handle.await {
            tracing::error!("Failed to join on statistics controller handle: {e}");
        }

        if let Err(e) = self.gateway_cache_join_handle.await {
            tracing::error!("Failed to join on gateway cache handle: {e}");
        }

        if let Err(e) = self.discovery_refresher_join_handle.await {
            tracing::error!("Failed to join on discovery refresher handle: {e}");
        }

        tracing::info!("Exiting vpn service run loop");

        Ok(())
    }

    async fn set_target_state(&mut self, new_state: TargetState) -> bool {
        if self.target_state != new_state || self.tunnel_state.read().await.is_error_state() {
            tracing::debug!("Set target state {} => {}", self.target_state, new_state);
            self.target_state = new_state;

            match new_state {
                TargetState::Secured => {
                    self.statistics_event_sender.report_connection_request();
                    let _ = self.command_sender.send(TunnelCommand::Connect);
                }
                TargetState::Unsecured => {
                    self.statistics_event_sender.report_disconnection_request();
                    let _ = self.command_sender.send(TunnelCommand::Disconnect);
                }
            }

            true
        } else {
            false
        }
    }

    async fn reconnect_tunnel(&self) -> bool {
        match self.target_state {
            TargetState::Secured => {
                self.statistics_event_sender.report_connection_request();
                let _ = self.command_sender.send(TunnelCommand::Connect);
                true
            }
            TargetState::Unsecured => false,
        }
    }

    fn update_tunnel_settings(&self) {
        let tunnel_settings = self.config_manager.generate_tunnel_settings();
        self.command_sender
            .send(TunnelCommand::SetTunnelSettings(tunnel_settings))
            .ok();
    }

    fn update_tunnel_settings_with_throttle(&mut self) {
        match self.target_state {
            TargetState::Secured => {
                let timer = tokio::time::sleep(Duration::from_secs(1)).fuse();
                self.tunnel_settings_update_timer.set(timer);
            }
            TargetState::Unsecured => self.update_tunnel_settings(),
        }
    }

    fn handle_tunnel_event(&mut self, event: TunnelEvent) {
        if let TunnelEvent::NewState(ref new_state) = event {
            if let Ok(mut state) = self.tunnel_state.try_write() {
                *state = new_state.clone();
            } else {
                tracing::error!("Failed to update tunnel state to {new_state}");
            }

            // Auto-disable SOCKS5 when VPN disconnects
            if matches!(new_state, TunnelState::Disconnected | TunnelState::Error(_))
                && self.socks5_service.is_enabled()
            {
                tracing::info!("VPN disconnected, auto-disabling SOCKS5 proxy");
                let socks5_service = self.socks5_service.clone();
                tokio::spawn(async move {
                    if let Err(e) = socks5_service.disable().await {
                        tracing::error!("Failed to auto-disable SOCKS5 on VPN disconnect: {}", e);
                    }
                });
            }
        }
        if self.tunnel_event_tx.send(event).is_err() {
            tracing::error!("Failed to send tunnel event");
        }
    }

    fn handle_account_state_change(&mut self, account_state: AccountControllerState) {
        if self
            .tunnel_event_tx
            .send(TunnelEvent::AccountState(account_state))
            .is_err()
        {
            tracing::error!("Failed to send tunnel event");
        }
    }

    fn handle_discovery_refresher_event(&mut self, event: DiscoveryRefresherEvent) {
        match event {
            DiscoveryRefresherEvent::NewNetwork(new_network) => {
                tracing::info!("Network environment updated");
                let _ = self.network_tx.send_replace(new_network);
            }
            DiscoveryRefresherEvent::Error(_error) => {
                // todo: handle error?
            }
        }
    }

    // Wrap handle_service_command in timing code to log long-running commands
    async fn handle_service_command_timed(&mut self, command: VpnServiceCommand) {
        let start = Instant::now();
        let command_str = command.to_string();
        self.handle_service_command(command).await;
        let elapsed = start.elapsed();
        if elapsed.as_millis() > 100 {
            tracing::warn!("{command_str} took {} ms to execute", elapsed.as_millis());
        }
    }

    async fn handle_service_command(&mut self, command: VpnServiceCommand) {
        match command {
            VpnServiceCommand::Info(tx, ()) => {
                let result = self.handle_info().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetConfig(tx, ()) => {
                let result = self.handle_get_config().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::SetEntryPoint(tx, entry_point) => {
                self.handle_set_entry_point(entry_point).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetExitPoint(tx, exit_point) => {
                self.handle_set_exit_point(exit_point).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetDisableIPv6(tx, disable_ipv6) => {
                self.handle_set_disable_ipv6(disable_ipv6).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetEnableTwoHop(tx, enable_two_hop) => {
                self.handle_set_enable_two_hop(enable_two_hop).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetNetstack(tx, netstack) => {
                self.handle_set_netstack(netstack).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetAllowLan(tx, allow_lan) => {
                self.handle_set_allow_lan(allow_lan).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetEnableBridges(tx, enable_bridges) => {
                self.handle_set_enable_bridges(enable_bridges).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetResidentialExit(tx, residential_exit) => {
                self.handle_set_residential_exit(residential_exit).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetEnableCustomDns(tx, enable_custom_dns) => {
                self.handle_set_enable_custom_dns(enable_custom_dns).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetCustomDns(tx, custom_dns) => {
                self.handle_set_custom_dns(custom_dns).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetNetwork(tx, network) => {
                let result = self.handle_set_network(network).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetSystemMessages(tx, ()) => {
                let result = self.handle_get_system_messages().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetNetworkCompatibility(tx, ()) => {
                let result = self.handle_get_network_compatibility().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetFeatureFlags(tx, ()) => {
                let result = self.handle_get_feature_flags().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetDefaultDns(tx, ()) => {
                let result = self.handle_get_default_dns().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::ListGateways(tx, options) => {
                self.handle_list_gateways(options, tx).await;
            }
            VpnServiceCommand::ListFilteredGateways(tx, filters) => {
                self.handle_list_filtered_gateways(filters, tx).await;
            }
            VpnServiceCommand::SetTargetState(tx, target_state) => {
                let accepted = self.set_target_state(target_state).await;
                let _ = tx.send(accepted);
            }
            VpnServiceCommand::Reconnect(tx, ()) => {
                let accepted = self.reconnect_tunnel().await;
                let _ = tx.send(accepted);
            }
            VpnServiceCommand::GetTunnelState(tx, ()) => {
                let result = self.handle_get_tunnel_state().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::StoreAccount(tx, account) => {
                let _ = tx.send(self.handle_store_account(account).await);
            }
            VpnServiceCommand::DecentralisedBalance(tx, ()) => {
                let _ = tx.send(self.handle_decentralised_balance().await);
            }
            VpnServiceCommand::DecentralisedObtainTicketbooks(tx, request) => {
                let _ = tx.send(self.handle_decentralised_obtain_ticketbooks(request).await);
            }
            VpnServiceCommand::IsAccountStored(tx, ()) => {
                let _ = tx.send(self.handle_is_account_stored().await);
            }
            VpnServiceCommand::ForgetAccount(tx, ()) => {
                let _ = tx.send(self.handle_forget_account().await);
            }
            VpnServiceCommand::RotateKeys(tx, ()) => {
                let _ = tx.send(self.handle_rotate_keys().await);
            }
            VpnServiceCommand::GetAccountIdentity(tx, ()) => {
                let _ = tx.send(self.handle_get_account_identity().await);
            }
            VpnServiceCommand::GetAccountLinks(tx, locale) => {
                let _ = tx.send(self.handle_get_account_links(locale).await);
            }
            VpnServiceCommand::GetAccountState(tx, ()) => {
                let _ = tx.send(self.handle_get_account_state().await);
            }
            VpnServiceCommand::RefreshAccountState(tx, ()) => {
                self.handle_refresh_account_state().await;
                let _ = tx.send(());
            }
            VpnServiceCommand::GetAccountUsage(tx, ()) => {
                let _ = tx.send(self.handle_get_usage().await);
            }
            VpnServiceCommand::ResetDeviceIdentity(tx, seed) => {
                let _ = tx.send(self.handle_reset_device_identity(seed).await);
            }
            VpnServiceCommand::GetDeviceIdentity(tx, ()) => {
                let _ = tx.send(self.handle_get_device_identity().await);
            }
            VpnServiceCommand::GetDevices(tx, ()) => {
                let _ = tx.send(self.handle_get_devices().await);
            }
            VpnServiceCommand::GetActiveDevices(tx, ()) => {
                let _ = tx.send(self.handle_get_active_devices().await);
            }
            VpnServiceCommand::GetAvailableTickets(tx, ()) => {
                let _ = tx.send(self.handle_get_available_tickets().await);
            }
            VpnServiceCommand::GetLogPath(tx, ()) => {
                let _ = tx.send(self.log_path.clone());
            }
            VpnServiceCommand::DeleteLogFile(tx, ()) => {
                self.handle_delete_log_file().await;
                let _ = tx.send(());
            }
            VpnServiceCommand::IsSentryEnabled(tx, ()) => {
                let enabled = self.handle_is_sentry_enabled().await;
                let _ = tx.send(enabled);
            }
            VpnServiceCommand::ToggleSentry(tx, enable) => {
                let result = self.handle_toggle_sentry(enable).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::EnableNetStats(tx, enabled) => {
                self.handle_enable_network_stats(enabled).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::AllowDisconnectedNetStats(tx, allow_disconnected) => {
                self.handle_allow_disconnected_network_stats(allow_disconnected)
                    .await;
                let _ = tx.send(());
            }
            VpnServiceCommand::ResetNetStatsSeed(tx, seed) => {
                let result = self.handle_reset_network_stats_seed(seed).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetNetStatsSeed(tx, ()) => {
                let identity = self.handle_get_network_stats_seed().await;
                let _ = tx.send(identity);
            }
            VpnServiceCommand::EnableSocks5(tx, enable_socks5_request) => {
                let result = self.handle_enable_socks5(enable_socks5_request).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::DisableSocks5(tx, ()) => {
                let result = self.handle_disable_socks5().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetSocks5Status(tx, ()) => {
                let result = self.handle_get_socks5_status().await;
                let _ = tx.send(result);
            }
        }
    }

    async fn handle_info(&self) -> VpnServiceInfo {
        let bin_info = nym_bin_common::bin_info_local_vergen!();
        let network_env = self.network_tx.borrow();

        VpnServiceInfo {
            version: bin_info.build_version.to_string(),
            build_timestamp: OffsetDateTime::parse(bin_info.build_timestamp, &Rfc3339).ok(),
            triple: bin_info.cargo_triple.to_string(),
            platform: self.user_agent.platform.clone(),
            git_commit: bin_info.commit_sha.to_string(),
            nym_network: NymNetworkDetails::from(network_env.nym_network.clone()),
            nym_vpn_network: NymVpnNetwork::from(network_env.nym_vpn_network.clone()),
        }
    }

    async fn handle_get_config(&self) -> VpnServiceConfig {
        self.config_manager.config().clone()
    }

    async fn handle_set_entry_point(&mut self, entry_point: EntryPoint) {
        self.config_manager.set_entry_point(entry_point).await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_exit_point(&mut self, exit_point: ExitPoint) {
        self.config_manager.set_exit_point(exit_point).await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_disable_ipv6(&mut self, disable_ipv6: bool) {
        self.config_manager.set_disable_ipv6(disable_ipv6).await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_enable_two_hop(&mut self, enable_two_hop: bool) {
        self.config_manager.set_enable_two_hop(enable_two_hop).await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_netstack(&mut self, netstack: bool) {
        self.config_manager.set_netstack(netstack).await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_allow_lan(&mut self, allow_lan: bool) {
        self.config_manager.set_allow_lan(allow_lan).await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_enable_bridges(&mut self, enable_bridges: bool) {
        self.config_manager.set_enable_bridges(enable_bridges).await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_residential_exit(&mut self, residential_exit: bool) {
        self.config_manager
            .set_residential_exit(residential_exit)
            .await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_enable_custom_dns(&mut self, enable_custom_dns: bool) {
        if self
            .config_manager
            .set_enable_custom_dns(enable_custom_dns)
            .await
        {
            let config = self.config_manager.config();
            // Ignore reconnect if custom DNS is enabled but custom DNS addresses aren't set
            if !enable_custom_dns || !config.custom_dns.is_empty() {
                self.update_tunnel_settings_with_throttle();
            }
        }
    }

    async fn handle_set_custom_dns(&mut self, mut custom_dns: Vec<IpAddr>) {
        const MAX_CUSTOM_DNS_SERVERS: usize = 5;

        if custom_dns.len() > MAX_CUSTOM_DNS_SERVERS {
            tracing::warn!("Only the first {MAX_CUSTOM_DNS_SERVERS} DNS servers will be used");
            custom_dns.truncate(MAX_CUSTOM_DNS_SERVERS);
        }

        if self.config_manager.set_custom_dns(custom_dns).await {
            let config = self.config_manager.config();
            // Only issue reconnect if custom DNS is enabled
            if config.enable_custom_dns {
                self.update_tunnel_settings_with_throttle();
            }
        }
    }

    async fn handle_set_network(&self, network: String) -> Result<(), SetNetworkError> {
        let mut global_config =
            GlobalConfig::read_from_default_config_dir()
                .await
                .map_err(|source| SetNetworkError::ReadConfig {
                    source: source.into(),
                })?;

        let network_selected = NetworkEnvironments::try_from(network.as_str())
            .map_err(|_err| SetNetworkError::NetworkNotFound(network.to_owned()))?;
        global_config.network_name = network_selected.to_string();

        global_config
            .write_to_default_config_dir()
            .await
            .map_err(|source| SetNetworkError::WriteConfig {
                source: source.into(),
            })?;

        tracing::info!(
            "Network updated to: {} (SERVICE RESTART REQUIRED!)",
            network_selected
        );
        Ok(())
    }

    async fn handle_get_system_messages(&self) -> Vec<SystemMessage> {
        self.network_tx
            .borrow()
            .nym_vpn_network
            .system_messages
            .messages
            .iter()
            .cloned()
            .map(SystemMessage::from)
            .collect()
    }

    async fn handle_get_network_compatibility(&self) -> Option<NetworkCompatibility> {
        self.network_tx
            .borrow()
            .system_configuration
            .as_ref()
            .and_then(|sc| sc.min_supported_app_versions.clone())
            .map(NetworkCompatibility::from)
    }

    async fn handle_get_feature_flags(&self) -> Option<FeatureFlags> {
        self.network_tx
            .borrow()
            .feature_flags
            .clone()
            .map(FeatureFlags::from)
    }

    async fn handle_get_default_dns(&self) -> Vec<IpAddr> {
        DEFAULT_DNS_SERVERS.clone()
    }

    async fn handle_list_gateways(
        &self,
        options: ListGatewaysOptions,
        completion_tx: oneshot::Sender<Result<Vec<Gateway>, ListGatewaysError>>,
    ) {
        let gateway_client = self.gateway_cache_handle.clone();

        tokio::spawn(async move {
            // todo: pass options.user_agent with request
            let result = gateway_client
                .lookup_gateways(nym_gateway_directory::GatewayType::from(options.gw_type))
                .await
                .map_err(|source| ListGatewaysError::GetGateways {
                    gw_type: options.gw_type,
                    source,
                })
                .map(|gateways| {
                    gateways
                        .into_iter()
                        .map(nym_vpn_lib_types::Gateway::from)
                        .collect::<Vec<_>>()
                });

            completion_tx.send(result).ok();
        });
    }

    async fn handle_list_filtered_gateways(
        &self,
        filters: GatewayFilters,
        completion_tx: oneshot::Sender<Result<Vec<Gateway>, ListGatewaysError>>,
    ) {
        let gateway_client = self.gateway_cache_handle.clone();
        let gw_type = filters.gw_type;

        tokio::spawn(async move {
            let result = gateway_client
                .lookup_filtered_gateways(nym_gateway_directory::GatewayFilters::from(filters))
                .await
                .map_err(|source| ListGatewaysError::GetFilteredGateways { gw_type, source })
                .map(|gateways| {
                    gateways
                        .into_iter()
                        .map(nym_vpn_lib_types::Gateway::from)
                        .collect::<Vec<_>>()
                });

            completion_tx.send(result).ok();
        });
    }

    async fn handle_enable_socks5(
        &mut self,
        enable_socks5_request: EnableSocks5Request,
    ) -> Result<(), Socks5Error> {
        tracing::info!("Enabling SOCKS5 client: {:?}", enable_socks5_request);

        // Get all exit gateways
        let exit_gateways: nym_gateway_directory::GatewayList = self
            .gateway_cache_handle
            .lookup_gateways(gateway_directory::GatewayType::MixnetExit)
            .await
            .map_err(|e| {
                Socks5Error::InvalidConfig(format!("Failed to lookup exit gateways: {}", e))
            })?;

        // Filter for gateways that support SOCKS5
        let socks5_gateways = gateway_directory::GatewayList::new(
            Some(gateway_directory::GatewayType::MixnetExit),
            exit_gateways
                .into_iter()
                .filter(|gateway| {
                    gateway
                        .last_probe
                        .as_ref()
                        .and_then(|probe| probe.outcome.as_exit.as_ref())
                        .map(|exit_point| exit_point.can_connect)
                        .unwrap_or(false)
                })
                .collect(),
        );

        // Get exit node's identity depending on the exit point
        let exit_point = &enable_socks5_request.exit_point;
        let gateway_identity: NodeIdentity = match exit_point {
            // User has chosen a specific exit address (IPR address)
            ExitPoint::Address { address } => *address.gateway().inner(),
            // User has chosen a specific gateway identity
            ExitPoint::Gateway { identity } => *identity.inner(),
            // User has chosen a specific exit country, region, or random
            ExitPoint::Country { .. } | ExitPoint::Region { .. } | ExitPoint::Random => {
                // For non-specific exit points, select a gateway the same way the VPN does
                tracing::info!(
                    "Selecting SOCKS5 exit node for exit point: {:?}",
                    exit_point
                );

                // Convert to gateway_directory types for lookup
                let exit_point_dir = gateway_directory::ExitPoint::from(exit_point.clone());
                let residential_exit = self.config_manager.config().residential_exit;

                // Try to find a high-performance gateway first
                let selected_gateway = exit_point_dir
                    .lookup_gateway(
                        &socks5_gateways,
                        Some(gateway_directory::ScoreValue::High),
                        residential_exit,
                    )
                    .or_else(|err| {
                        // When no gateways could be found, lower performance tier and try again
                        if err.is_unmatched_non_specific_gateway() {
                            tracing::debug!(
                                "Could not locate high quality SOCKS5 exit gateway. \
                                Lowering performance filter to medium and trying again"
                            );
                            exit_point_dir.lookup_gateway(
                                &socks5_gateways,
                                Some(gateway_directory::ScoreValue::Medium),
                                residential_exit,
                            )
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|e| {
                        Socks5Error::InvalidConfig(format!("Failed to select exit gateway: {}", e))
                    })?;

                tracing::info!(
                    "Selected SOCKS5 exit gateway: {}, location: {}",
                    selected_gateway.identity(),
                    selected_gateway
                        .two_letter_iso_country_code()
                        .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
                );

                selected_gateway.identity()
            }
        };

        // Get the gateway with nr_address from cache (or fetch if not cached)
        let gateway = self
            .gateway_cache_handle
            .lookup_nymnode_by_identity(gateway_identity)
            .await
            .map_err(|e| {
                Socks5Error::InvalidConfig(format!(
                    "Failed to lookup gateway {}: {}",
                    gateway_identity, e
                ))
            })?;

        let nr_address = gateway
            .nr_address
            .as_ref()
            .ok_or(Socks5Error::GatewayNotSupported)?
            .clone();

        tracing::info!("Using network requester address {} for SOCKS5", nr_address);

        let request_timeout = socks5_request_timeout();
        let idle_timeout = socks5_idle_timeout();

        self.socks5_service
            .enable(
                self.data_dir.clone(),
                enable_socks5_request.socks5_settings.listen_address,
                enable_socks5_request.http_rpc_settings.listen_address,
                nr_address,
                request_timeout,
                idle_timeout,
            )
            .await?;

        tracing::info!("Lazy SOCKS5 proxy service enabled successfully");
        tracing::info!(
            "Mixnet will initialize on first SOCKS5 connection and shut down after {}s of inactivity",
            idle_timeout.as_secs()
        );
        Ok(())
    }

    async fn handle_disable_socks5(&mut self) -> Result<(), Socks5Error> {
        tracing::info!("Disabling lazy SOCKS5 proxy service");
        self.socks5_service.disable().await?;
        tracing::info!("Lazy SOCKS5 proxy service disabled successfully");
        Ok(())
    }

    async fn handle_get_socks5_status(&self) -> Result<Socks5Status, Socks5Error> {
        self.socks5_service.get_status().await
    }

    async fn handle_get_tunnel_state(&self) -> TunnelState {
        self.tunnel_state.read().await.clone()
    }

    async fn handle_store_account(
        &mut self,
        store_request: StoreAccountRequest,
    ) -> Result<(), AccountCommandError> {
        let mnemonic = nym_vpn_lib::login::parse_account_request(&store_request)
            .map_err(|err| AccountCommandError::InvalidSecret(err.to_string()))?;
        if store_request.centralised() {
            self.account_command_tx
                .store_account(StorableAccount::new(mnemonic, StoredAccountMode::Api))
                .await
        } else {
            self.account_command_tx
                .store_account(StorableAccount::new(
                    mnemonic,
                    StoredAccountMode::Decentralised,
                ))
                .await
        }
    }

    async fn handle_decentralised_balance(&mut self) -> AccountBalanceResponse {
        AccountBalanceResponse {
            result: self
                .account_command_tx
                .decentralised_balance()
                .await
                .map(|v| v.into_iter().map(nym_vpn_lib_types::Coin::from).collect()),
        }
    }

    async fn handle_decentralised_obtain_ticketbooks(
        &mut self,
        request: DecentralisedObtainTicketbooksRequest,
    ) -> Result<(), AccountCommandError> {
        let amount = request.amount;
        tracing::info!("received request to attempt to obtain {amount} ticketbooks of each type");

        self.account_command_tx
            .decentralised_obtain_ticketbooks(amount)
            .await
    }

    async fn handle_is_account_stored(&self) -> bool {
        self.account_command_tx
            .get_account_id()
            .await
            .map(|id| id.is_some())
            .unwrap_or(false)
    }

    async fn handle_forget_account(&mut self) -> Result<(), AccountCommandError> {
        if *self.tunnel_state.read().await != TunnelState::Disconnected {
            return Err(AccountCommandError::internal(
                "Unable to forget account while connected",
            ));
        }

        let data_dir = self.data_dir.clone();
        tracing::info!(
            "REMOVING ALL ACCOUNT AND DEVICE DATA IN: {}",
            data_dir.display()
        );

        let _ = self
            .stats_control_commands_sender
            .reset_seed(None)
            .await
            .inspect_err(|e| tracing::error!("Failed to reset networks stats seed: {e}"));

        self.account_command_tx.forget_account().await
    }

    async fn handle_rotate_keys(&mut self) -> Result<(), AccountCommandError> {
        // TODO: temporary, until key rotation can be done while connected
        if *self.tunnel_state.read().await != TunnelState::Disconnected {
            return Err(AccountCommandError::internal(
                "Unable to rotate keys while connected",
            ));
        }

        self.account_command_tx.rotate_keys().await
    }

    async fn handle_get_account_identity(&self) -> Result<Option<String>, AccountCommandError> {
        self.account_command_tx.get_account_id().await
    }

    async fn handle_get_account_links(
        &self,
        locale: String,
    ) -> Result<ParsedAccountLinks, AccountLinksError> {
        let account_id = self
            .handle_get_account_identity()
            .await
            .map_err(|_| AccountLinksError::FailedToParseAccountLinks)?;

        self.network_tx
            .borrow()
            .nym_vpn_network
            .account_management
            .clone()
            .ok_or(AccountLinksError::AccountManagementNotConfigured)?
            .try_into_parsed_links(&locale, account_id.as_deref())
            .map(ParsedAccountLinks::from)
            .map_err(|err| {
                tracing::error!("Failed to parse account links: {:?}", err);
                AccountLinksError::FailedToParseAccountLinks
            })
    }

    async fn handle_get_account_state(&self) -> AccountControllerState {
        self.account_state_rx.get_state()
    }

    async fn handle_refresh_account_state(&self) {
        let _ = self
            .account_command_tx
            .background_refresh_account_state()
            .await;
    }

    async fn handle_get_usage(&self) -> Result<Vec<NymVpnUsage>, AccountCommandError> {
        self.account_command_tx
            .get_usage()
            .await
            .map(|s| s.into_iter().map(NymVpnUsage::from).collect())
    }

    async fn handle_reset_device_identity(
        &mut self,
        seed: Option<[u8; 32]>,
    ) -> Result<(), AccountCommandError> {
        if *self.tunnel_state.read().await != TunnelState::Disconnected {
            return Err(AccountCommandError::internal(
                "Unable to reset device identity while connected",
            ));
        }

        self.account_command_tx.reset_device_identity(seed).await?;

        let _ = self
            .stats_control_commands_sender
            .reset_seed(None)
            .await
            .inspect_err(|e| tracing::error!("Failed to reset networks stats seed: {e}"));

        Ok(())
    }

    async fn handle_get_device_identity(&self) -> Result<Option<String>, AccountCommandError> {
        self.account_command_tx.get_device_identity().await
    }

    async fn handle_get_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        Ok(self
            .account_command_tx
            .get_devices()
            .await?
            .into_iter()
            .map(NymVpnDevice::from)
            .collect())
    }

    async fn handle_get_active_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        Ok(self
            .account_command_tx
            .get_active_devices()
            .await?
            .into_iter()
            .map(NymVpnDevice::from)
            .collect())
    }

    async fn handle_get_available_tickets(
        &self,
    ) -> Result<AvailableTicketbooks, AccountCommandError> {
        self.account_command_tx.get_available_tickets().await
    }

    async fn handle_delete_log_file(&self) {
        if let Some(remove_log_file_handle) = self.log_file_remover_handle.as_ref() {
            remove_log_file_handle.remove_log_file();
        }
    }

    async fn handle_is_sentry_enabled(&self) -> bool {
        GlobalConfig::read_from_default_config_dir()
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to read global config file: {}", e);
            })
            .ok()
            .map(|c| c.sentry_monitoring)
            // if something goes wrong with the config file, fallback to the real state of Sentry client
            .unwrap_or(self.sentry_enabled)
    }

    async fn handle_toggle_sentry(&self, enable: bool) -> Result<(), GlobalConfigError> {
        let mut config = GlobalConfig::read_from_default_config_dir()
            .await
            .map_err(|e| GlobalConfigError::ReadConfig(e.to_string()))?;
        config.sentry_monitoring = enable;
        if enable {
            tracing::info!("Sentry monitoring enabled, daemon needs to be restarted");
        } else {
            if let Some(client) = sentry::Hub::current().client() {
                client.close(Some(Duration::from_secs(1)));
                tracing::debug!("Sentry client closed");
            }
            tracing::info!("Sentry monitoring disabled, daemon needs to be restarted");
        }
        GlobalConfig::write_to_default_config_dir(&config)
            .await
            .map_err(|e| GlobalConfigError::WriteConfig(e.to_string()))?;
        Ok(())
    }

    async fn handle_enable_network_stats(&mut self, enabled: bool) {
        self.config_manager.set_netstats_enabled(enabled).await;
        self.stats_control_commands_sender
            .set_enable_collection(enabled);
    }

    async fn handle_allow_disconnected_network_stats(&mut self, allow_disconnected: bool) {
        self.config_manager
            .set_netstats_allow_disconnected(allow_disconnected)
            .await;
        self.stats_control_commands_sender
            .set_allow_direct_sending(allow_disconnected);
    }

    async fn handle_reset_network_stats_seed(
        &mut self,
        seed: Option<String>,
    ) -> Result<(), StatisticsControllerError> {
        self.stats_control_commands_sender.reset_seed(seed).await
    }

    async fn handle_get_network_stats_seed(
        &mut self,
    ) -> Result<NetworkStatisticsIdentity, StatisticsControllerError> {
        self.stats_control_commands_sender.get_seed().await
    }
}
