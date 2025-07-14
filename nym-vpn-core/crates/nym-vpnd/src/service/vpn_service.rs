// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, path::PathBuf, sync::Arc, time::Instant};

use bip39::Mnemonic;
use nym_statistics::{
    config::StatisticsControllerConfig,
    controller::StatisticsController,
    events::{StatisticsEvent, StatisticsSender},
};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::{
    sync::{broadcast, mpsc, oneshot, watch},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use nym_vpn_account_controller::{
    AccountCommandSender, AccountController, AccountControllerConfig, AccountStateSummary,
    AvailableTicketbooks, SharedAccountState,
};
use nym_vpn_api_client::{
    NetworkCompatibility,
    response::{NymVpnDevice, NymVpnUsage},
    types::{Percent, ScoreThresholds},
};
use nym_vpn_lib::{
    MixnetClientConfig, UserAgent, VpnTopologyProvider,
    gateway_directory::{
        self, CachingGatewayClient, EntryPoint, ExitPoint, GatewayClient, GatewayType,
    },
    tunnel_state_machine::{
        DnsOptions, GatewayPerformanceOptions, MixnetTunnelOptions, NymConfig, TunnelCommand,
        TunnelSettings, TunnelStateMachine, WireguardMultihopMode, WireguardTunnelOptions,
    },
};
use nym_vpn_lib_types::{
    AccountCommandError, ForgetAccountError, StoreAccountError, TunnelEvent, TunnelState,
    TunnelType,
};
use nym_vpn_network_config::{FeatureFlags, Network, ParsedAccountLinks, SystemMessages};
use nym_vpnd_types::{
    gateway::{Country, Gateway},
    service::{VpnServiceConnectError, VpnServiceDisconnectError, VpnServiceInfo},
};
use std::time::Duration;
use zeroize::Zeroizing;

use super::{
    config::{DEFAULT_CONFIG_FILE, NetworkEnvironments, NymVpnServiceConfig},
    error::{
        AccountControllerError, AccountLinksError, Error, GlobalConfigError, ListGatewaysError,
        Result, SetNetworkError, StatisticsControllerError, VpnServiceDeleteLogFileError,
    },
};
use crate::{config::GlobalConfigFile, logging::LogPath};

// Seed used to generate device identity keys
type Seed = [u8; 32];

type Locale = String;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, strum::Display)]
pub enum VpnServiceCommand {
    Info(oneshot::Sender<VpnServiceInfo>, ()),
    SetNetwork(oneshot::Sender<Result<(), SetNetworkError>>, String),
    GetSystemMessages(oneshot::Sender<SystemMessages>, ()),
    GetNetworkCompatibility(oneshot::Sender<Option<NetworkCompatibility>>, ()),
    GetFeatureFlags(oneshot::Sender<Option<FeatureFlags>>, ()),
    ListGateways(
        oneshot::Sender<Result<Vec<Gateway>, ListGatewaysError>>,
        ListGatewaysOptions,
    ),
    ListCountries(
        oneshot::Sender<Result<Vec<Country>, ListGatewaysError>>,
        ListCountriesOptions,
    ),
    Connect(
        oneshot::Sender<Result<(), VpnServiceConnectError>>,
        ConnectArgs,
    ),
    Disconnect(oneshot::Sender<Result<(), VpnServiceDisconnectError>>, ()),
    GetTunnelState(oneshot::Sender<TunnelState>, ()),
    SubscribeToTunnelState(oneshot::Sender<watch::Receiver<TunnelState>>, ()),
    StoreAccount(
        oneshot::Sender<Result<(), StoreAccountError>>,
        Zeroizing<String>,
    ),
    IsAccountStored(oneshot::Sender<bool>, ()),
    ForgetAccount(oneshot::Sender<Result<(), ForgetAccountError>>, ()),
    GetAccountIdentity(oneshot::Sender<Option<String>>, ()),
    GetAccountLinks(
        oneshot::Sender<Result<ParsedAccountLinks, AccountLinksError>>,
        Locale,
    ),
    GetAccountState(oneshot::Sender<AccountStateSummary>, ()),
    RefreshAccountState(oneshot::Sender<()>, ()),
    GetAccountUsage(
        oneshot::Sender<Result<Vec<NymVpnUsage>, AccountCommandError>>,
        (),
    ),
    ResetDeviceIdentity(
        oneshot::Sender<Result<(), AccountCommandError>>,
        Option<Seed>,
    ),
    GetDeviceIdentity(oneshot::Sender<Result<String, AccountCommandError>>, ()),
    RegisterDevice(oneshot::Sender<()>, ()),
    GetDevices(
        oneshot::Sender<Result<Vec<NymVpnDevice>, AccountCommandError>>,
        (),
    ),
    GetActiveDevices(
        oneshot::Sender<Result<Vec<NymVpnDevice>, AccountCommandError>>,
        (),
    ),
    RequestZkNym(oneshot::Sender<()>, ()),
    GetDeviceZkNyms(oneshot::Sender<Result<(), AccountCommandError>>, ()),
    GetZkNymsAvailableForDownload(oneshot::Sender<Result<(), AccountCommandError>>, ()),
    GetZkNymById(oneshot::Sender<Result<(), AccountCommandError>>, String),
    ConfirmZkNymIdDownloaded(oneshot::Sender<Result<(), AccountCommandError>>, String),
    GetAvailableTickets(
        oneshot::Sender<Result<AvailableTicketbooks, AccountCommandError>>,
        (),
    ),
    GetLogPath(oneshot::Sender<Option<LogPath>>, ()),
    DeleteLogFile(
        oneshot::Sender<Result<(), VpnServiceDeleteLogFileError>>,
        (),
    ),
    IsSentryEnabled(oneshot::Sender<bool>, ()),
    ToggleSentry(oneshot::Sender<Result<(), GlobalConfigError>>, bool),
}

#[derive(Debug)]
pub struct ListGatewaysOptions {
    pub gw_type: GatewayType,
    #[allow(unused)]
    pub user_agent: Option<UserAgent>,
}

#[derive(Debug)]
pub struct ListCountriesOptions {
    pub gw_type: GatewayType,
    #[allow(unused)]
    pub user_agent: Option<UserAgent>,
}

#[derive(Debug)]
pub struct ConnectArgs {
    pub entry: Option<gateway_directory::EntryPoint>,
    pub exit: Option<gateway_directory::ExitPoint>,
    pub options: ConnectOptions,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ConnectOptions {
    pub dns: Option<IpAddr>,
    pub enable_two_hop: bool,
    pub netstack: bool,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub enable_credentials_mode: bool,
    pub min_mixnode_performance: Option<Percent>,
    pub min_gateway_mixnet_performance: Option<Percent>,
    pub min_gateway_vpn_performance: Option<Percent>,
    pub user_agent: Option<UserAgent>,
}

pub struct NymVpnService<S>
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

    // Broadcast channel for sending tunnel events to the outside world
    tunnel_event_tx: broadcast::Sender<TunnelEvent>,

    // Send command to delete and recreate logging file
    file_logging_event_tx: mpsc::Sender<()>,

    // Send commands to the account controller
    account_command_tx: AccountCommandSender,

    // Path to the main config file
    config_file: PathBuf,

    // Path to the data directory
    data_dir: PathBuf,

    // If log to file is enabled, path to the log directory and log filename
    log_path: Option<LogPath>,

    // Storage backend
    storage: Arc<tokio::sync::Mutex<S>>,

    // Last known tunnel state wrapped in a `watch::Sender` that can be used to track tunnel state individually
    tunnel_state: watch::Sender<TunnelState>,

    // Tunnel state machine handle
    state_machine_handle: JoinHandle<()>,

    // Account controller handle
    account_controller_handle: JoinHandle<()>,

    // Command channel for state machine
    command_sender: mpsc::UnboundedSender<TunnelCommand>,

    // Event channel for receiving events from state machine
    event_receiver: mpsc::UnboundedReceiver<TunnelEvent>,

    // Service shutdown token.
    shutdown_token: CancellationToken,

    // Gateway directory client
    gateway_directory_client: CachingGatewayClient,

    // Sentry client has been initialized and is enabled
    sentry_enabled: bool,

    // The statistics channel sender
    statistics_event_sender: StatisticsSender,
}

impl NymVpnService<nym_vpn_lib::storage::VpnClientOnDiskStorage> {
    #![allow(clippy::too_many_arguments)]
    pub fn spawn(
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        tunnel_event_tx: broadcast::Sender<TunnelEvent>,
        file_logging_event_tx: mpsc::Sender<()>,
        shutdown_token: CancellationToken,
        network_env: Network,
        user_agent: UserAgent,
        stats_id_seed: Option<String>,
        log_path: Option<LogPath>,
        sentry_enabled: bool,
    ) -> JoinHandle<()> {
        tracing::trace!("Starting VPN service");
        tokio::spawn(async move {
            match NymVpnService::new(
                vpn_command_rx,
                tunnel_event_tx,
                file_logging_event_tx,
                shutdown_token,
                network_env,
                user_agent,
                stats_id_seed,
                log_path,
                sentry_enabled,
            )
            .await
            {
                Ok(service) => {
                    tracing::debug!("VPN service initialized successfully");

                    match service.run().await {
                        Ok(_) => {
                            tracing::info!("VPN service has successfully exited");
                        }
                        Err(e) => {
                            tracing::error!("VPN service has exited with error: {e:?}");
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("Failed to initialize VPN service: {err:?}");
                }
            }
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        tunnel_event_tx: broadcast::Sender<TunnelEvent>,
        file_logging_event_tx: mpsc::Sender<()>,
        shutdown_token: CancellationToken,
        network_env: Network,
        user_agent: UserAgent,
        stats_id_seed: Option<String>,
        log_path: Option<LogPath>,
        sentry_enabled: bool,
    ) -> Result<Self> {
        let network_name = network_env.nym_network_details().network_name.clone();

        let config_dir = super::config::config_dir().join(&network_name);
        let config_file = config_dir.join(DEFAULT_CONFIG_FILE);
        let data_dir = super::config::data_dir();
        let network_data_dir = data_dir.join(&network_name);

        let storage = Arc::new(tokio::sync::Mutex::new(
            nym_vpn_lib::storage::VpnClientOnDiskStorage::new(network_data_dir.clone()),
        ));

        // Make sure the data dir exists
        super::config::create_data_dir(&data_dir, &network_name).map_err(Error::ConfigSetup)?;

        let statistics_api = network_env
            .system_configuration
            .as_ref()
            .and_then(|config| config.statistics_api.clone());

        let account_controller_config = AccountControllerConfig {
            data_dir: network_data_dir.clone(),
            user_agent: user_agent.clone(),
            credentials_mode: None,
            network_env: network_env.clone(),
        };

        let route_handler = nym_vpn_lib::tunnel_state_machine::RouteHandler::new()
            .await
            .map_err(nym_vpn_lib::tunnel_state_machine::Error::CreateRouteHandler)
            .map_err(Error::StateMachine)?;

        let connectivity_handle = nym_offline_monitor::spawn_monitor(
            route_handler.inner_handle(),
            #[cfg(target_os = "linux")]
            Some(nym_vpn_lib::tunnel_state_machine::TUNNEL_FWMARK),
        )
        .await;

        let account_controller = AccountController::new(
            account_controller_config,
            Arc::clone(&storage),
            Some(connectivity_handle.clone()),
            shutdown_token.child_token(),
        )
        .await
        .map_err(|err| {
            tracing::error!("Failed to create account controller: {err:?}");
            AccountControllerError::Initialization {
                reason: err.to_string(),
            }
        })?;

        let tunnel_state = watch::Sender::new(TunnelState::Disconnected);

        // These are used to interact with the account controller
        let shared_account_state = account_controller.get_shared_state();
        let account_command_tx = account_controller.get_command_sender();
        let account_controller_handle = tokio::task::spawn(account_controller.run());

        // Statistics collection setup
        let statistics_controller_config =
            StatisticsControllerConfig::new(statistics_api, user_agent.clone())
                .with_stats_id_seed(stats_id_seed);

        // Statistics collection can technically fail, but if it's the case, we just disable it as it is not operation critical.
        let statistics_controller = StatisticsController::new(
            statistics_controller_config,
            network_data_dir.clone(),
            shutdown_token.child_token(),
            tunnel_state.subscribe(),
        )
        .await;

        let statistics_event_sender = statistics_controller.get_statistics_sender();
        let _statistics_controller_handle = tokio::task::spawn(statistics_controller.run());

        // These used to interact with the tunnel state machine
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let tunnel_settings = TunnelSettings::default();
        let nyxd_url = network_env.nyxd_url();
        let api_url = network_env.api_url();

        let mix_score_thresholds =
            network_env
                .system_configuration
                .as_ref()
                .map(|sc| ScoreThresholds {
                    high: sc.mix_thresholds.high,
                    medium: sc.mix_thresholds.medium,
                    low: sc.mix_thresholds.low,
                });
        let wg_score_thresholds =
            network_env
                .system_configuration
                .as_ref()
                .map(|sc| ScoreThresholds {
                    high: sc.wg_thresholds.high,
                    medium: sc.wg_thresholds.medium,
                    low: sc.wg_thresholds.low,
                });

        let gateway_config = gateway_directory::Config {
            nyxd_url,
            api_url,
            nym_vpn_api_url: Some(network_env.vpn_api_url()),
            min_gateway_performance: None,
            mix_score_thresholds,
            wg_score_thresholds,
        };
        let nym_config = NymConfig {
            config_path: Some(config_dir),
            data_path: Some(network_data_dir.clone()),
            gateway_config: gateway_config.clone(),
            network_env: network_env.clone(),
        };

        let gateway_directory_client =
            GatewayClient::new(gateway_config, user_agent.clone()).unwrap();
        let gateway_directory_client =
            CachingGatewayClient::new(gateway_directory_client, Some(connectivity_handle.clone()));
        gateway_directory_client.refresh_all().await;

        let topology_provider = VpnTopologyProvider::new(
            network_env.api_url(),
            Some(user_agent.clone()),
            false,
            shutdown_token.child_token(),
        );
        topology_provider.fetch().await;

        let state_machine_handle = TunnelStateMachine::spawn(
            command_receiver,
            event_sender,
            nym_config,
            tunnel_settings,
            account_command_tx.clone(),
            gateway_directory_client.clone(),
            topology_provider,
            connectivity_handle,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            route_handler,
            shutdown_token.child_token(),
        )
        .await
        .map_err(Error::StateMachine)?;

        Ok(Self {
            network_env,
            user_agent,
            shared_account_state,
            vpn_command_rx,
            tunnel_event_tx,
            file_logging_event_tx,
            account_command_tx,
            config_file,
            data_dir: network_data_dir,
            log_path,
            storage,
            tunnel_state,
            state_machine_handle,
            account_controller_handle,
            command_sender,
            event_receiver,
            shutdown_token,
            gateway_directory_client,
            sentry_enabled,
            statistics_event_sender,
        })
    }
}

impl<S> NymVpnService<S>
where
    S: nym_vpn_store::VpnStorage,
{
    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                Some(command) = self.vpn_command_rx.recv() => {
                    self.handle_service_command_timed(command).await;
                }
                Some(event) = self.event_receiver.recv() => {
                    if let Err(e) = self.tunnel_event_tx.send(event.clone()) {
                        tracing::error!("Failed to send tunnel event: {}", e);
                    }

                    match event {
                        TunnelEvent::NewState(new_state) => {
                            // Replace value even when there are no receivers.
                            let _ = self.tunnel_state.send_replace(new_state.clone());
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

        if let Err(e) = self.account_controller_handle.await {
            tracing::error!("Failed to join on account controller handle: {}", e);
        }

        if let Err(e) = self.state_machine_handle.await {
            tracing::error!("Failed to join on state machine handle: {}", e);
        }

        tracing::info!("Exiting vpn service run loop");

        Ok(())
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
                self.handle_list_gateways(options, tx);
            }
            VpnServiceCommand::ListCountries(tx, options) => {
                self.handle_list_countries(options, tx)
            }
            VpnServiceCommand::Connect(tx, connect_args) => {
                let result = self.handle_connect(connect_args).await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::Disconnect(tx, ()) => {
                let result = self.handle_disconnect().await;
                let _ = tx.send(result);
            }
            VpnServiceCommand::GetTunnelState(tx, ()) => {
                let result = self.handle_get_tunnel_state();
                let _ = tx.send(result);
            }
            VpnServiceCommand::SubscribeToTunnelState(tx, ()) => {
                let rx = self.handle_subscribe_to_tunnel_state();
                let _ = tx.send(rx);
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
            VpnServiceCommand::RegisterDevice(tx, ()) => {
                self.handle_register_device().await;
                let _ = tx.send(());
            }
            VpnServiceCommand::GetDevices(tx, ()) => {
                let _ = tx.send(self.handle_get_devices().await);
            }
            VpnServiceCommand::GetActiveDevices(tx, ()) => {
                let _ = tx.send(self.handle_get_active_devices().await);
            }
            VpnServiceCommand::RequestZkNym(tx, ()) => {
                self.handle_request_zk_nym().await;
                let _ = tx.send(());
            }
            VpnServiceCommand::GetDeviceZkNyms(tx, ()) => {
                let _ = tx.send(self.handle_get_device_zk_nyms().await);
            }
            VpnServiceCommand::GetZkNymsAvailableForDownload(tx, ()) => {
                let _ = tx.send(self.handle_get_zk_nyms_available_for_download().await);
            }
            VpnServiceCommand::GetZkNymById(tx, id) => {
                let _ = tx.send(self.handle_get_zk_nym_by_id(id).await);
            }
            VpnServiceCommand::ConfirmZkNymIdDownloaded(tx, id) => {
                let _ = tx.send(self.handle_confirm_zk_nym_id_downloaded(id).await);
            }
            VpnServiceCommand::GetAvailableTickets(tx, ()) => {
                let _ = tx.send(self.handle_get_available_tickets().await);
            }
            VpnServiceCommand::GetLogPath(tx, ()) => {
                let _ = tx.send(self.log_path.clone());
            }
            VpnServiceCommand::DeleteLogFile(tx, ()) => {
                let _ = tx.send(self.handle_delete_log_file().await);
            }
            VpnServiceCommand::IsSentryEnabled(tx, ()) => {
                let _ = tx.send(self.handle_is_sentry_enabled().await);
            }
            VpnServiceCommand::ToggleSentry(tx, enable) => {
                let _ = tx.send(self.handle_toggle_sentry(enable).await);
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

    async fn handle_connect(
        &mut self,
        connect_args: ConnectArgs,
    ) -> Result<(), VpnServiceConnectError> {
        let ConnectArgs {
            entry,
            exit,
            mut options,
        } = connect_args;

        self.statistics_event_sender
            .report(StatisticsEvent::new_connecting(options.enable_two_hop));

        // Get feature flag
        let enable_credentials_mode = self
            .network_env
            .get_feature_flag_credential_mode()
            .unwrap_or(false);
        tracing::debug!("feature flag: credential mode: {enable_credentials_mode}");

        options.enable_credentials_mode =
            options.enable_credentials_mode || enable_credentials_mode;

        tracing::debug!(
            "Using entry point: {}",
            entry
                .clone()
                .map(|e| e.to_string())
                .unwrap_or("None".to_string())
        );
        tracing::debug!(
            "Using exit point: {}",
            exit.clone()
                .map(|e| e.to_string())
                .unwrap_or("None".to_string())
        );
        tracing::debug!("Using options: {:?}", options);

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
            mixnet_tunnel_options: MixnetTunnelOptions {
                mtu: None,
                enable_credentials_mode,
            },
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
                VpnServiceDisconnectError::Internal("failed to send disconnect command".to_owned())
            })
    }

    fn handle_get_tunnel_state(&self) -> TunnelState {
        self.tunnel_state.borrow().to_owned()
    }

    fn handle_subscribe_to_tunnel_state(&self) -> watch::Receiver<TunnelState> {
        self.tunnel_state.subscribe()
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

    async fn handle_set_network(&self, network: String) -> Result<(), SetNetworkError> {
        let mut global_config =
            GlobalConfigFile::read_from_file().map_err(|source| SetNetworkError::ReadConfig {
                source: source.into(),
            })?;

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

    async fn handle_get_network_compatibility(&self) -> Option<NetworkCompatibility> {
        self.network_env
            .system_configuration
            .as_ref()
            .and_then(|sc| sc.min_supported_app_versions.clone())
    }

    async fn handle_get_feature_flags(&self) -> Option<FeatureFlags> {
        self.network_env.feature_flags.clone()
    }

    fn handle_list_gateways(
        &self,
        options: ListGatewaysOptions,
        completion_tx: oneshot::Sender<Result<Vec<Gateway>, ListGatewaysError>>,
    ) {
        let gateway_client = self.gateway_directory_client.clone();

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

    fn handle_list_countries(
        &self,
        options: ListCountriesOptions,
        completion_tx: oneshot::Sender<Result<Vec<Country>, ListGatewaysError>>,
    ) {
        let gateway_client = self.gateway_directory_client.clone();

        tokio::spawn(async move {
            // todo: pass options.user_agent with request
            let result = gateway_client
                .lookup_countries(options.gw_type)
                .await
                .map_err(|source| ListGatewaysError::GetCountries {
                    gw_type: options.gw_type,
                    source,
                })
                .map(|countries| {
                    countries
                        .into_iter()
                        .map(nym_vpnd_types::gateway::Country::from)
                        .collect::<Vec<_>>()
                });
            completion_tx.send(result).ok();
        });
    }

    async fn handle_store_account(
        &mut self,
        account: Zeroizing<String>,
    ) -> Result<(), StoreAccountError> {
        let mnemonic = Mnemonic::parse::<&str>(account.as_ref())
            .map_err(|err| StoreAccountError::InvalidMnemonic(err.to_string()))?;
        self.account_command_tx.store_account(mnemonic).await
    }

    async fn handle_is_account_stored(&self) -> bool {
        self.shared_account_state.is_account_stored().await
    }

    async fn handle_forget_account(&mut self) -> Result<(), ForgetAccountError> {
        if *self.tunnel_state.borrow() != TunnelState::Disconnected {
            return Err(ForgetAccountError::internal(
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

    async fn handle_get_account_identity(&self) -> Option<String> {
        self.shared_account_state.get_account_id().await
    }

    async fn handle_get_account_links(
        &self,
        locale: String,
    ) -> Result<ParsedAccountLinks, AccountLinksError> {
        let account_id = self.handle_get_account_identity().await;

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

    async fn handle_get_account_state(&self) -> AccountStateSummary {
        self.shared_account_state.lock().await.clone()
    }

    async fn handle_refresh_account_state(&self) {
        self.account_command_tx.background_sync_account_state();
    }

    async fn handle_get_usage(&self) -> Result<Vec<NymVpnUsage>, AccountCommandError> {
        self.account_command_tx.get_usage().await
    }

    async fn handle_reset_device_identity(
        &mut self,
        seed: Option<[u8; 32]>,
    ) -> Result<(), AccountCommandError> {
        if *self.tunnel_state.borrow() != TunnelState::Disconnected {
            return Err(AccountCommandError::internal(
                "Unable to reset device identity while connected",
            ));
        }

        self.storage
            .lock()
            .await
            .reset_keys(seed)
            .await
            .map_err(|err| AccountCommandError::Storage(err.to_string()))?;

        self.statistics_event_sender
            .report(StatisticsEvent::reset_seed());

        self.account_command_tx.background_sync_account_state();

        Ok(())
    }

    async fn handle_get_device_identity(&self) -> Result<String, AccountCommandError> {
        self.account_command_tx.get_device_identity().await
    }

    async fn handle_register_device(&self) {
        self.account_command_tx.background_sync_device_state();
    }

    async fn handle_get_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        self.account_command_tx.get_devices().await
    }

    async fn handle_get_active_devices(&self) -> Result<Vec<NymVpnDevice>, AccountCommandError> {
        self.account_command_tx.get_active_devices().await
    }

    async fn handle_request_zk_nym(&self) {
        self.account_command_tx.background_request_zk_nyms();
    }

    async fn handle_get_device_zk_nyms(&self) -> Result<(), AccountCommandError> {
        self.account_command_tx.get_device_zk_nym()
    }

    async fn handle_get_zk_nyms_available_for_download(&self) -> Result<(), AccountCommandError> {
        self.account_command_tx.get_zk_nyms_available_for_download()
    }

    async fn handle_get_zk_nym_by_id(&self, id: String) -> Result<(), AccountCommandError> {
        self.account_command_tx.get_zk_nym_by_id(id)
    }

    async fn handle_confirm_zk_nym_id_downloaded(
        &self,
        id: String,
    ) -> Result<(), AccountCommandError> {
        self.account_command_tx.confirm_zk_nym_id_downloaded(id)
    }

    async fn handle_get_available_tickets(
        &self,
    ) -> Result<AvailableTicketbooks, AccountCommandError> {
        self.account_command_tx.get_available_tickets().await
    }

    async fn handle_delete_log_file(&self) -> Result<(), VpnServiceDeleteLogFileError> {
        match self.file_logging_event_tx.try_send(()) {
            Ok(_) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                tracing::debug!("Already trying to delete file");
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                tracing::error!("Failed to send command to delete log file: channel is closed");
                return Err(VpnServiceDeleteLogFileError::Internal(
                    "failed to send delete log command".to_owned(),
                ));
            }
        }
        Ok(())
    }

    async fn handle_is_sentry_enabled(&self) -> bool {
        GlobalConfigFile::read_from_file()
            .inspect_err(|e| {
                tracing::error!("Failed to read global config file: {}", e);
            })
            .ok()
            .map(|c| c.sentry_monitoring)
            // if something goes wrong with the config file, fallback to the real state of Sentry client
            .unwrap_or(self.sentry_enabled)
    }

    async fn handle_toggle_sentry(&self, enable: bool) -> Result<(), GlobalConfigError> {
        let mut config = GlobalConfigFile::read_from_file()
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
        GlobalConfigFile::write_to_file(&config)
            .map_err(|e| GlobalConfigError::WriteConfig(e.to_string()))?;
        Ok(())
    }
}
