// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bip39::Mnemonic;
use futures::{FutureExt, StreamExt, future::Fuse, pin_mut};
use nym_statistics::{
    StatisticsController, StatisticsControllerConfig,
    events::{StatisticsEvent, StatisticsSender},
};
use std::{path::PathBuf, pin::Pin};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
    time::{Duration, Instant},
};
use tokio_stream::wrappers::WatchStream;
use tokio_util::sync::CancellationToken;

use nym_common::trace_err_chain;
use nym_registration_client::MixnetClientConfig;
use nym_vpn_account_controller::{
    AccountCommandSender, AccountController, AccountControllerConfig, AccountStateReceiver,
    AvailableTicketbooks,
};
use nym_vpn_api_client::{
    NetworkCompatibility,
    response::{NymVpnDevice, NymVpnUsage},
    types::ScoreThresholds,
};
use nym_vpn_lib::{
    UserAgent, VpnTopologyProvider,
    gateway_directory::{
        self, EntryPoint, ExitPoint, GatewayCache, GatewayCacheHandle, GatewayClient,
    },
    tunnel_state_machine::{NymConfig, TunnelCommand, TunnelConstants, TunnelStateMachine},
};
use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerState, TunnelEvent, TunnelState, VpnServiceConfig,
};
use nym_vpn_network_config::{FeatureFlags, Network, ParsedAccountLinks, SystemMessages};
use nym_vpnd_types::{
    ListGatewaysOptions, StoreAccountRequest,
    gateway::Gateway,
    log_path::LogPath,
    service::{ConnectArgs, TargetState, VpnServiceInfo},
};

use super::{
    config::{NetworkEnvironments, VpnServiceConfigManager},
    error::{
        AccountControllerError, AccountLinksError, Error, GlobalConfigError, ListGatewaysError,
        Result, SetNetworkError,
    },
};
use crate::{config::GlobalConfig, logging::LogFileRemoverHandle};

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
    SetNetwork(oneshot::Sender<Result<(), SetNetworkError>>, String),
    GetSystemMessages(oneshot::Sender<SystemMessages>, ()),
    GetNetworkCompatibility(oneshot::Sender<Option<NetworkCompatibility>>, ()),
    GetFeatureFlags(oneshot::Sender<Option<FeatureFlags>>, ()),
    ListGateways(
        oneshot::Sender<Result<Vec<Gateway>, ListGatewaysError>>,
        ListGatewaysOptions,
    ),
    // Deprecated
    Connect(oneshot::Sender<()>, ConnectArgs),
    SetTargetState(oneshot::Sender<bool>, TargetState),
    Reconnect(oneshot::Sender<bool>, ()),
    GetTunnelState(oneshot::Sender<TunnelState>, ()),
    StoreAccount(
        oneshot::Sender<Result<(), AccountCommandError>>,
        StoreAccountRequest,
    ),
    IsAccountStored(oneshot::Sender<bool>, ()),
    ForgetAccount(oneshot::Sender<Result<(), AccountCommandError>>, ()),
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
    IsCollectNetStatsEnabled(oneshot::Sender<bool>, ()),
    ToggleCollectNetStats(oneshot::Sender<Result<(), GlobalConfigError>>, bool),
}

pub struct NymVpnServiceParameters {
    pub log_path: Option<LogPath>,
    pub network_env: Box<Network>,
    pub sentry_enabled: bool,
    pub netstats_enabled: bool,
    pub stats_id_seed: Option<String>,
    pub user_agent: UserAgent,
}

pub struct NymVpnService {
    // The network environment
    network_env: Box<Network>,

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
    tunnel_state: TunnelState,

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

    // VPN service shutdown token.
    shutdown_token: CancellationToken,

    // Shutdown token used by state machine
    state_machine_shutdown_token: CancellationToken,

    // Shutdown token used for account and statistics controllers and other services that are safe to exit altogether.
    services_shutdown_token: CancellationToken,

    // Sentry client has been initialized and is enabled
    sentry_enabled: bool,

    // Whether network statistics reporting is enabled
    network_statistics_enabled: bool,

    // The statistics channel sender
    statistics_event_sender: StatisticsSender,
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
            let Ok(service) = NymVpnService::new(
                vpn_command_rx,
                tunnel_event_tx,
                log_file_remover_handle,
                parameters,
                shutdown_token,
            )
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

        let statistics_api = parameters
            .network_env
            .system_configuration
            .as_ref()
            .and_then(|config| config.statistics_api.clone());

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
        )
        .map_err(|err| {
            trace_err_chain!(err, "Failed to create NymVPN API client");
            AccountControllerError::Initialization {
                reason: err.to_string(),
            }
        })?;

        let account_controller = AccountController::new(
            nym_vpn_api_client,
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
        let account_controller_handle = tokio::task::spawn(account_controller.run());

        // Statistics collection setup
        let statistics_controller_config =
            StatisticsControllerConfig::new(statistics_api, parameters.user_agent.clone())
                .with_stats_id_seed(parameters.stats_id_seed)
                .with_enabled(parameters.netstats_enabled);

        // Statistics collection can technically fail, but if it's the case, we just disable it as it is not operation critical.
        let statistics_controller = StatisticsController::new(
            statistics_controller_config,
            network_data_dir.clone(),
            services_shutdown_token.child_token(),
        )
        .await;

        let config_manager =
            VpnServiceConfigManager::new(&config_dir, Some(tunnel_event_tx.clone())).await?;

        let statistics_event_sender = statistics_controller.get_statistics_sender();
        let statistics_controller_handle = tokio::task::spawn(statistics_controller.run());

        // These used to interact with the tunnel state machine
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let tunnel_settings = config_manager.generate_tunnel_settings();
        let nyxd_url = parameters.network_env.nyxd_url();
        let api_url = parameters.network_env.api_url();

        let mix_score_thresholds = parameters
            .network_env
            .system_configuration
            .as_ref()
            .map(|sc| ScoreThresholds {
                high: sc.mix_thresholds.high,
                medium: sc.mix_thresholds.medium,
                low: sc.mix_thresholds.low,
            });
        let wg_score_thresholds = parameters
            .network_env
            .system_configuration
            .as_ref()
            .map(|sc| ScoreThresholds {
                high: sc.wg_thresholds.high,
                medium: sc.wg_thresholds.medium,
                low: sc.wg_thresholds.low,
            });

        let gateway_config = gateway_directory::Config {
            nyxd_url,
            api_url: api_url.clone(),
            nym_vpn_api_url: Some(parameters.network_env.vpn_api_url()),
            min_gateway_performance: None,
            mix_score_thresholds,
            wg_score_thresholds,
        };
        let nym_config = NymConfig {
            config_path: Some(config_dir),
            data_path: Some(network_data_dir.clone()),
            gateway_config: gateway_config.clone(),
            network_env: *parameters.network_env.clone(),
        };

        let gateway_directory_client =
            GatewayClient::new(gateway_config, parameters.user_agent.clone()).unwrap();
        let (gateway_cache_handle, gateway_cache_join_handle) = GatewayCache::spawn(
            gateway_directory_client,
            connectivity_handle.clone(),
            services_shutdown_token.child_token(),
        );

        let validator_client = nym_http_api_client::Client::builder(api_url)
            .map_err(Box::new)?
            .with_user_agent(parameters.user_agent.clone())
            .build()
            .map_err(Box::new)?;
        let topology_provider = VpnTopologyProvider::new(
            parameters.network_env.api_url(),
            validator_client,
            false,
            services_shutdown_token.child_token(),
        );
        topology_provider.fetch().await;

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
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_handler,
            state_machine_shutdown_token.child_token(),
        )
        .await
        .map_err(Error::StateMachine)?;

        Ok(Self {
            network_env: parameters.network_env,
            user_agent: parameters.user_agent,
            vpn_command_rx,
            tunnel_event_tx,
            log_file_remover_handle,
            account_command_tx,
            account_state_rx,
            data_dir: network_data_dir,
            log_path: parameters.log_path,
            target_state: TargetState::Unsecured,
            tunnel_state: TunnelState::Disconnected,
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
            sentry_enabled: parameters.sentry_enabled,
            network_statistics_enabled: parameters.netstats_enabled,
            statistics_event_sender,
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

        if let Err(e) = self.account_controller_handle.await {
            tracing::error!("Failed to join on account controller handle: {}", e);
        }

        if let Err(e) = self.statistics_controller_handle.await {
            tracing::error!("Failed to join on statistics controller handle: {}", e);
        }

        if let Err(e) = self.gateway_cache_join_handle.await {
            tracing::error!("Failed to join on gateway cache handle: {}", e);
        }

        tracing::info!("Exiting vpn service run loop");

        Ok(())
    }

    async fn set_target_state(&mut self, new_state: TargetState) -> bool {
        if self.target_state != new_state || self.tunnel_state.is_error_state() {
            tracing::debug!("Set target state {} => {}", self.target_state, new_state);
            self.target_state = new_state;

            match new_state {
                TargetState::Secured => {
                    let _ = self.command_sender.send(TunnelCommand::Connect);
                }
                TargetState::Unsecured => {
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
        if let TunnelEvent::NewState(ref state) = event {
            self.tunnel_state = state.clone();
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
                self.handle_set_allow_lan(allow_lan, tx).await;
            }
            VpnServiceCommand::SetEnableBridges(tx, enable_bridges) => {
                self.handle_set_enable_bridges(enable_bridges).await;
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
            VpnServiceCommand::ListGateways(tx, options) => {
                self.handle_list_gateways(options, tx).await;
            }
            VpnServiceCommand::Connect(tx, connect_args) => {
                self.handle_connect(connect_args).await.ok();
                let _ = tx.send(());
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
            VpnServiceCommand::IsAccountStored(tx, ()) => {
                let _ = tx.send(self.handle_is_account_stored().await);
            }
            VpnServiceCommand::ForgetAccount(tx, ()) => {
                let _ = tx.send(self.handle_forget_account().await);
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
            VpnServiceCommand::IsCollectNetStatsEnabled(tx, ()) => {
                let _ = tx.send(self.handle_is_collect_network_stats_enabled().await);
            }
            VpnServiceCommand::ToggleCollectNetStats(tx, enable) => {
                let result = self.handle_toggle_collect_network_stats(enable).await;
                let _ = tx.send(result);
            }
        }
    }

    async fn handle_info(&self) -> VpnServiceInfo {
        let bin_info = nym_bin_common::bin_info_local_vergen!();

        VpnServiceInfo {
            version: bin_info.build_version.to_string(),
            build_timestamp: OffsetDateTime::parse(bin_info.build_timestamp, &Rfc3339).ok(),
            triple: bin_info.cargo_triple.to_string(),
            platform: self.user_agent.platform.clone(),
            git_commit: bin_info.commit_sha.to_string(),
            nym_network: self.network_env.nym_network.clone(),
            nym_vpn_network: self.network_env.nym_vpn_network.clone(),
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

    async fn handle_set_allow_lan(&mut self, allow_lan: bool, complete_tx: oneshot::Sender<()>) {
        self.config_manager.set_allow_lan(allow_lan).await;
        _ = self
            .command_sender
            .send(TunnelCommand::SetAllowLan(allow_lan, complete_tx));
    }

    async fn handle_set_enable_bridges(&mut self, enable_bridges: bool) {
        self.config_manager.set_enable_bridges(enable_bridges).await;
        self.update_tunnel_settings_with_throttle();
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

    async fn handle_get_system_messages(&self) -> SystemMessages {
        self.network_env.nym_vpn_network.system_messages.clone()
    }

    async fn handle_get_network_compatibility(&self) -> Option<NetworkCompatibility> {
        self.network_env
            .system_configuration
            .as_ref()
            .and_then(|sc| sc.min_supported_app_versions.clone())
    }

    async fn handle_get_feature_flags(&self) -> Option<FeatureFlags> {
        self.network_env.feature_flags.clone()
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
                .lookup_gateways(options.gw_type)
                .await
                .map_err(|source| ListGatewaysError::GetGateways {
                    gw_type: options.gw_type,
                    source,
                })
                .map(|gateways| {
                    gateways
                        .into_iter()
                        .map(nym_vpnd_types::gateway::Gateway::from)
                        .collect::<Vec<_>>()
                });

            completion_tx.send(result).ok();
        });
    }

    // Deprecated
    async fn handle_connect(&mut self, connect_args: ConnectArgs) -> Result<()> {
        let ConnectArgs {
            entry,
            exit,
            options,
        } = connect_args;

        let entry_point = entry.unwrap_or(self.config_manager.config().entry_point.clone());
        let exit_point = exit.unwrap_or(self.config_manager.config().exit_point.clone());
        let config = VpnServiceConfig {
            entry_point,
            exit_point,
            disable_ipv6: options.disable_ipv6,
            enable_two_hop: options.enable_two_hop,
            enable_bridges: false,
            netstack: options.netstack,
            dns: options.dns,
            allow_lan: true, // always true to support legacy behavior
            min_mixnode_performance: None,
            min_gateway_mixnet_performance: None,
            min_gateway_vpn_performance: None,
            disable_poisson_rate: options.disable_poisson_rate,
            disable_background_cover_traffic: options.disable_background_cover_traffic,
        };

        self.config_manager.set_config(config).await;

        self.statistics_event_sender
            .report(StatisticsEvent::new_connecting(
                self.config_manager.config().enable_two_hop,
            ));

        self.update_tunnel_settings();

        // Ensure to always reconnect to maintain the legacy behavior
        if self.target_state == TargetState::Secured {
            let _ = self.command_sender.send(TunnelCommand::Connect);
        } else {
            let _ = self.set_target_state(TargetState::Secured).await;
        }

        Ok(())
    }

    async fn handle_get_tunnel_state(&self) -> TunnelState {
        self.tunnel_state.clone()
    }

    async fn handle_store_account(
        &mut self,
        store_request: StoreAccountRequest,
    ) -> Result<(), AccountCommandError> {
        let mnemonic = Mnemonic::parse::<&str>(store_request.mnemonic.as_str())
            .map_err(|err| AccountCommandError::InvalidMnemonic(err.to_string()))?;
        self.account_command_tx.store_account(mnemonic).await
    }

    async fn handle_is_account_stored(&self) -> bool {
        self.account_command_tx
            .get_account_id()
            .await
            .map(|id| id.is_some())
            .unwrap_or(false)
    }

    async fn handle_forget_account(&mut self) -> Result<(), AccountCommandError> {
        if self.tunnel_state != TunnelState::Disconnected {
            return Err(AccountCommandError::internal(
                "Unable to forget account while connected",
            ));
        }

        let data_dir = self.data_dir.clone();
        tracing::info!(
            "REMOVING ALL ACCOUNT AND DEVICE DATA IN: {}",
            data_dir.display()
        );

        self.statistics_event_sender
            .report(StatisticsEvent::remove_seed());

        self.account_command_tx.forget_account().await
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

        self.network_env
            .nym_vpn_network
            .account_management
            .clone()
            .ok_or(AccountLinksError::AccountManagementNotConfigured)?
            .try_into_parsed_links(&locale, account_id.as_deref())
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
        self.account_command_tx.get_usage().await
    }

    async fn handle_reset_device_identity(
        &mut self,
        seed: Option<[u8; 32]>,
    ) -> Result<(), AccountCommandError> {
        if self.tunnel_state != TunnelState::Disconnected {
            return Err(AccountCommandError::internal(
                "Unable to reset device identity while connected",
            ));
        }

        self.account_command_tx.reset_device_identity(seed).await?;

        self.statistics_event_sender
            .report(StatisticsEvent::reset_seed());

        Ok(())
    }

    async fn handle_get_device_identity(&self) -> Result<Option<String>, AccountCommandError> {
        self.account_command_tx.get_device_identity().await
    }

    async fn handle_get_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        self.account_command_tx.get_devices().await
    }

    async fn handle_get_active_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        self.account_command_tx.get_active_devices().await
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

    async fn handle_is_collect_network_stats_enabled(&self) -> bool {
        self.network_statistics_enabled
    }

    async fn handle_toggle_collect_network_stats(
        &mut self,
        enable: bool,
    ) -> Result<(), GlobalConfigError> {
        let mut config = GlobalConfig::read_from_default_config_dir()
            .await
            .map_err(|e| GlobalConfigError::ReadConfig(e.to_string()))?;
        config.collect_network_statistics = enable;
        if enable {
            tracing::info!("Collect network statistics enabled, daemon needs to be restarted");
        } else {
            tracing::info!("Collect network statistics disabled, daemon needs to be restarted");
        }
        GlobalConfig::write_to_default_config_dir(&config)
            .await
            .map_err(|e| GlobalConfigError::WriteConfig(e.to_string()))?;
        self.network_statistics_enabled = enable;
        Ok(())
    }
}
