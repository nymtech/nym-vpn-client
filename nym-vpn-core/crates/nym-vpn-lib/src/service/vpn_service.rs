// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashSet,
    net::IpAddr,
    pin::{Pin, pin},
    sync::Arc,
};

#[cfg(target_os = "linux")]
use std::path::PathBuf;

use futures::{
    FutureExt, StreamExt,
    future::{Fuse, FusedFuture},
};
use nym_bandwidth_controller::BandwidthController;
use nym_diagnostic::DiagnosticHandler;
use nym_sdk::mixnet::StoragePaths;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::{
    sync::{RwLock, broadcast, mpsc, oneshot, watch},
    task::JoinHandle,
    time::{Duration, Instant},
};
use tokio_stream::wrappers::WatchStream;
use tokio_util::sync::CancellationToken;

use nym_common::trace_err_chain;
use nym_gateway_directory::{GatewayFilter, GatewayFilters};
use nym_statistics::{
    StatisticsCommandsSender, StatisticsController, StatisticsControllerError, StatisticsSender,
};
use nym_vpn_account_controller::{
    AccountCommandSender, AccountController, AccountControllerConfig, AccountStateReceiver,
    AvailableTicketbooks, NyxdClient,
};
use nym_vpn_api_client::api_urls_to_urls;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use nym_vpn_lib_types::ErrorStateReason;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use nym_vpn_lib_types::SplitApp;
#[cfg(target_os = "linux")]
use nym_vpn_lib_types::SplitTunnelExcludedProcess;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use nym_vpn_lib_types::SplitTunnelExcludedProcessList;
use nym_vpn_lib_types::{
    AccountBalanceResponse, AccountCommandError, AccountControllerState, AutologinResponse,
    DecentralisedObtainTicketbooksRequest, DeeplinkClient, DeeplinkKind, DiagnosticRegisterParams,
    DiagnosticReport, DiagnosticRunParams, EnableSocks5Request, EntryPoint, ExitPoint,
    FeatureFlags, Gateway, GatewaySelectionAlgorithm, GetDeeplinkParams, ListGatewaysOptions,
    LogPath, LookupGatewayFilters, MixnetTrafficConfig, NetworkCompatibility,
    NetworkStatisticsIdentity, NymNetworkDetails, NymVpnDevice, NymVpnNetwork, NymVpnUsage,
    ParsedAccountLinks, RegistrationReport, StorableAccount, StoreAccountRequest,
    StoredAccountMode, SystemMessage, TargetState, TentativeGateways, TunnelEvent, TunnelState,
    VpnAccountSummary, VpnServiceConfig, VpnServiceInfo,
};
#[cfg(any(target_os = "android", target_os = "ios"))]
use nym_vpn_lib_types::{RegisterAccountRequest, RegisterAccountResponse};
use nym_vpn_network_config::{DiscoveryRefresher, Network, NetworkCache};

use super::{
    Socks5Error, Socks5Service, Socks5Status,
    config::{NetworkEnvironments, VpnServiceConfigManager},
    error::{
        AccountLinksError, Error, GeoExclusionConfigError, GlobalConfigError, ListGatewaysError,
        Result, SetNetworkError,
    },
    socks5::Socks5EnableConfig,
    socks5_idle_timeout, socks5_request_timeout,
};
#[cfg(target_os = "android")]
use crate::tunnel_provider::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::OSTunProvider;
#[cfg(target_os = "linux")]
use crate::tunnel_state_machine::LinuxSplitTunnelConfiguration;
use crate::{
    DEFAULT_DNS_SERVERS_CONFIG, NodeIdentity, UserAgent, VpnTopologyService,
    config::GlobalConfig,
    gateway_directory::{self, GatewayCache, GatewayCacheHandle, GatewayClient},
    logging::LogFileRemoverHandle,
    paths::{NymConfigPaths, Paths},
    tunnel_state_machine::{
        NymConfig, TunnelCommand, TunnelConstants, TunnelStateMachine,
        tunnel::gateway_provider::GatewayProvider,
    },
};

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
    SetEnableAdBlocking(oneshot::Sender<()>, bool),
    SetFrontingMode(oneshot::Sender<()>, nym_vpn_lib_types::FrontingMode),
    SetResidentialExit(oneshot::Sender<()>, bool),
    SetEnableCustomDns(oneshot::Sender<()>, bool),
    SetCustomDns(oneshot::Sender<()>, Vec<IpAddr>),
    SetMixnetTrafficConfig(oneshot::Sender<Result<(), String>>, MixnetTrafficConfig),
    SetGatewaySelectionAlgorithm(oneshot::Sender<()>, GatewaySelectionAlgorithm),
    SetEnableGeoLocation(oneshot::Sender<Result<(), String>>, bool),
    SetEnableGatewayIndependence(oneshot::Sender<()>, bool),
    SetGatewayIndependenceNotifications(oneshot::Sender<()>, bool),
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
        LookupGatewayFilters,
    ),
    SetGeoExclusionEnabled(oneshot::Sender<()>, bool),
    SetGeoExclusionListenPort(oneshot::Sender<Result<(), GeoExclusionConfigError>>, u16),
    SetGeoExclusionExcludedCountries(
        oneshot::Sender<Result<(), GeoExclusionConfigError>>,
        Vec<String>,
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
    #[cfg(any(target_os = "android", target_os = "ios"))]
    RegisterAccount(
        oneshot::Sender<Result<RegisterAccountResponse, AccountCommandError>>,
        RegisterAccountRequest,
    ),
    CreateAccount(oneshot::Sender<Result<(), AccountCommandError>>, ()),
    StoreAccount(
        oneshot::Sender<Result<(), AccountCommandError>>,
        StoreAccountRequest,
    ),
    GetStoredMnemonic(oneshot::Sender<Result<String, AccountCommandError>>, ()),
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
    GetCanonicalAccountIdentity(
        oneshot::Sender<Result<Option<String>, AccountCommandError>>,
        (),
    ),
    GetAccountMode(oneshot::Sender<Option<StoredAccountMode>>, ()),
    GetAccountLinks(
        oneshot::Sender<Result<ParsedAccountLinks, AccountLinksError>>,
        Locale,
    ),
    GetAccountState(oneshot::Sender<AccountControllerState>, ()),
    RefreshAccountState(oneshot::Sender<()>, bool),
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
    GetAccountSummary(
        oneshot::Sender<Result<Option<VpnAccountSummary>, AccountCommandError>>,
        (),
    ),
    HandleSubscriptionPayment(oneshot::Sender<Result<(), AccountCommandError>>, ()),
    GetDeeplink(
        oneshot::Sender<Result<String, AccountCommandError>>,
        GetDeeplinkParams,
    ),
    GetAutologinDeeplink(
        oneshot::Sender<Result<AutologinResponse, AccountCommandError>>,
        GetDeeplinkParams,
    ),
    DeeplinkStoreAccount(oneshot::Sender<Result<(), AccountCommandError>>, String),
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
    RunDiagnostic(oneshot::Sender<DiagnosticReport>, DiagnosticRunParams),
    RegisterDiagnostic(
        oneshot::Sender<RegistrationReport>,
        DiagnosticRegisterParams,
    ),
    GetTentativeGateways(oneshot::Sender<TentativeGateways>, ()),
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    IsSplitTunnelSupported(oneshot::Sender<bool>, ()),
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    SetEnableSplitTunnel(oneshot::Sender<()>, bool),
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    AddSplitTunnelApp(oneshot::Sender<()>, SplitApp),
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    RemoveSplitTunnelApp(oneshot::Sender<()>, SplitApp),
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    ClearSplitTunnelApps(oneshot::Sender<()>, ()),
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    GetSplitTunnelExcludedProcesses(oneshot::Sender<SplitTunnelExcludedProcessList>, ()),
    #[cfg(target_os = "linux")]
    AddSplitTunnelProcess(oneshot::Sender<()>, i32),
    #[cfg(target_os = "linux")]
    RemoveSplitTunnelProcess(oneshot::Sender<()>, i32),
    #[cfg(target_os = "linux")]
    ClearSplitTunnelProcesses(oneshot::Sender<()>, ()),
    #[cfg(target_os = "macos")]
    NeedFullDiskPermissions(oneshot::Sender<bool>, ()),
}

/// Type of service configuration storage used by the VPN service.
pub enum ServiceConfigStorageType {
    /// Ephemeral in-memory configuration with the given initial value.
    Ephemeral(Box<VpnServiceConfig>),

    /// Persistent configuration that reads and writes service configuration from disk.
    Persistent,
}

pub struct NymVpnServiceParameters {
    pub paths: Paths,
    pub network_cache: NetworkCache,
    pub sentry_enabled: bool,
    pub user_agent: UserAgent,
    pub service_storage_type: ServiceConfigStorageType,
    #[cfg(target_os = "ios")]
    pub tun_provider: Arc<dyn OSTunProvider>,
    #[cfg(target_os = "android")]
    pub tun_provider: Arc<dyn AndroidTunProvider>,
    #[cfg(target_os = "android")]
    pub connectivity_monitor: Box<dyn nym_offline_monitor::NativeConnectivityAdapter + 'static>,
}

pub struct NymVpnService {
    // Paths
    paths: NymConfigPaths,

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

    // Broadcast channel for sending tunnel events to the outside world
    tunnel_event_tx: broadcast::Sender<TunnelEvent>,

    // Target state
    target_state: TargetState,

    // Last known tunnel state
    tunnel_state: Arc<RwLock<TunnelState>>,

    // Timer used to throttle changes to tunnel settings
    tunnel_settings_update_timer: Pin<Box<Fuse<tokio::time::Sleep>>>,

    // Command channel for state machine
    command_sender: Arc<mpsc::UnboundedSender<TunnelCommand>>,

    // Event channel for receiving events from state machine
    event_receiver: mpsc::UnboundedReceiver<TunnelEvent>,

    // Tunnel state machine handle
    state_machine_handle: Option<JoinHandle<()>>,

    // Account controller handle
    account_controller_handle: JoinHandle<()>,

    // Statistics controller handle
    statistics_controller_handle: JoinHandle<()>,

    // Bandwidth controller handle
    bandwidth_controller_handle: JoinHandle<()>,

    // Topology service join handle
    topology_service_join_handle: JoinHandle<()>,

    // Topology service handle
    topology_service_handle: crate::VpnTopologyServiceHandle,

    // Configuration Manager
    config_manager: VpnServiceConfigManager,

    // Gateway cache join handle
    gateway_cache_join_handle: JoinHandle<()>,

    // Gateway cache handle
    gateway_cache_handle: GatewayCacheHandle,

    // Discovery refresher event receiver
    discovery_refresher_event_rx: mpsc::UnboundedReceiver<Box<Network>>,

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

    // Split-tunnel management handle
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[cfg_attr(windows, allow(unused))]
    split_tunnel: nym_split_tunnel::SplitTunnelHandle,

    // Split-tunnel join handle
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    split_tunnel_join_handle: JoinHandle<()>,

    // Gateway provider
    gateway_provider: GatewayProvider<GatewayCacheHandle>,

    // Gateway provider join handle
    gateway_provider_handle: JoinHandle<()>,

    #[cfg(target_os = "linux")]
    split_tunnel_pid_manager: nym_split_tunnel::PidManager,
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

            match Box::pin(service.run()).await {
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
        let network_cache = parameters.network_cache;
        let network_env = network_cache
            .network()
            .map_err(|_| Error::NetworkEnvNotInitialized)?;

        let network_name = network_env.nym_network_details().network_name.clone();

        let paths = NymConfigPaths {
            data_dir: parameters.paths.data_dir.clone(),
            network_data_dir: parameters.paths.data_dir.join(&network_name),
            config_dir: parameters.paths.config_dir.clone(),
            log_dir: parameters.paths.log_dir.clone(),
            log_path: parameters.paths.log_path.clone(),
        };

        // Make sure the directories exist
        paths
            .create_directories()
            .await
            .map_err(Error::PathsSetup)?;

        let storage = crate::storage::VpnClientOnDiskStorage::new(paths.network_data_dir.clone());

        let state_machine_shutdown_token = CancellationToken::new();
        let services_shutdown_token = CancellationToken::new();

        #[cfg(target_os = "linux")]
        let routing_params = crate::tunnel_state_machine::RoutingParameters::default();

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let route_handler = crate::tunnel_state_machine::RouteHandler::new(
            #[cfg(target_os = "linux")]
            routing_params,
        )
        .await
        .map_err(crate::tunnel_state_machine::Error::CreateRouteHandler)
        .map_err(Error::StateMachine)?;

        let tunnel_constants = TunnelConstants::default();
        let connectivity_handle = nym_offline_monitor::spawn_monitor(
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_handler.inner_handle(),
            #[cfg(target_os = "linux")]
            Some(tunnel_constants.fwmark),
            #[cfg(target_os = "android")]
            parameters.connectivity_monitor,
        )
        .await;

        let account_controller_config = AccountControllerConfig {
            data_dir: paths.network_data_dir.clone(),
            network_env: *network_env.clone(),
        };

        let network_details = network_env.nym_network_details();
        let nym_vpn_api_client = nym_vpn_api_client::VpnApiClient::from_network(
            network_details,
            Some(parameters.user_agent.clone()),
        )
        .await
        .map_err(Error::CreateApiClient)?;

        let nyxd_client = NyxdClient::new(&network_env);

        let account_controller = AccountController::new(
            nym_vpn_api_client.clone(),
            nyxd_client,
            account_controller_config,
            storage,
            connectivity_handle.clone(),
            services_shutdown_token.child_token(),
        )
        .await
        .map_err(Error::CreateAccountController)?;

        // These are used to interact with the account controller
        let account_command_tx = account_controller.get_command_sender();
        let account_state_rx = account_controller.get_state_receiver();
        let wireguard_keys_db = account_controller.get_wireguard_keys_storage();
        let account_controller_handle = tokio::task::spawn(account_controller.run());

        let credential_storage = StoragePaths::new_from_dir(paths.network_data_dir.clone())
            .map_err(|e| Error::BandwidthControllerStorage(Box::new(e)))?
            .persistent_credential_storage()
            .await
            .map_err(|e| Error::BandwidthControllerStorage(Box::new(e)))?;
        let bandwidth_controller = BandwidthController::new(credential_storage);
        let bandwidth_command_tx = bandwidth_controller.get_request_sender();
        let bandwidth_controller_handle = tokio::task::spawn(
            bandwidth_controller.run(services_shutdown_token.child_token().into()),
        );

        let config_manager = match parameters.service_storage_type {
            ServiceConfigStorageType::Persistent => {
                VpnServiceConfigManager::new(&paths.config_dir, Some(tunnel_event_tx.clone()))
                    .await?
            }
            ServiceConfigStorageType::Ephemeral(initial_config) => {
                VpnServiceConfigManager::new_ephermeral(
                    initial_config,
                    Some(tunnel_event_tx.clone()),
                )
            }
        };

        // Statistics collection setup
        let statistics_controller_config = config_manager.config().network_stats;

        let statistics_api_url = network_env
            .system_configuration
            .as_ref()
            .and_then(|config| config.statistics_api.clone());

        let stats_api_client = statistics_api_url.and_then(|url| nym_statistics_api_client::StatisticsApiClient::new(url.clone(), parameters.user_agent.clone()).inspect_err(|e| tracing::error!("Failed to build Statistics API client. Statistics collection will be disabled : {e}")).ok());

        // Statistics collection can technically fail, but if it's the case, we just disable it as it is not operation critical.
        let statistics_controller = StatisticsController::new(
            statistics_controller_config,
            stats_api_client,
            paths.network_data_dir.clone(),
            services_shutdown_token.child_token(),
        )
        .await;

        let statistics_event_sender = statistics_controller.get_statistics_sender();
        let stats_control_commands_sender = statistics_controller.get_commands_sender();
        let statistics_controller_handle = tokio::task::spawn(statistics_controller.run());

        let tunnel_state = Arc::new(RwLock::new(TunnelState::Disconnected));

        // Initialize lazy SOCKS5 service (disabled by default)
        let socks5_service = Socks5Service::new(tunnel_state.clone());

        // Channels used for interacting with the tunnel state machine
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let command_sender = Arc::new(command_sender);
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let tunnel_settings = config_manager.generate_tunnel_settings();
        let nyxd_url = network_env.nyxd_url();

        let nym_api_urls = network_env
            .nym_api_urls()
            .ok_or(Error::InvalidEnvironment("empty nym_api_urls"))?;
        let nym_vpn_api_urls = network_env
            .nym_vpn_api_urls()
            .ok_or(Error::InvalidEnvironment("empty nym_api_urls"))?;

        let gateway_config =
            gateway_directory::Config::new(nyxd_url, nym_api_urls.clone(), nym_vpn_api_urls, None)
                .map_err(Error::CreateGatewayClient)?;

        let (network_tx, network_rx) = watch::channel(network_env.clone());
        let nym_config = NymConfig {
            paths: paths.clone(),
            gateway_config: gateway_config.clone(),
            network_rx,
        };

        let gateway_directory_client =
            GatewayClient::new(gateway_config, parameters.user_agent.clone())
                .map_err(Error::CreateGatewayClient)?;
        let (gateway_cache_handle, gateway_cache_join_handle) = GatewayCache::spawn(
            gateway_directory_client.clone(),
            connectivity_handle.clone(),
            services_shutdown_token.child_token(),
        );

        let urls = api_urls_to_urls(&nym_api_urls).map_err(Error::ConvertApiUrls)?;

        let (topology_service, topology_service_join_handle) = VpnTopologyService::spawn(
            urls.clone(),
            parameters.user_agent.clone(),
            services_shutdown_token.child_token(),
        );

        let cloned_topology_service = topology_service.clone();
        tokio::spawn(async move {
            let _ = cloned_topology_service.fetch().await;
        });

        let (discovery_refresher_event_tx, discovery_refresher_event_rx) =
            mpsc::unbounded_channel();
        let (discovery_refresher_command_tx, discovery_refresher_command_rx) =
            mpsc::unbounded_channel();
        let discovery_refresher_join_handle = DiscoveryRefresher::spawn(
            network_cache,
            discovery_refresher_command_rx,
            discovery_refresher_event_tx,
            connectivity_handle.clone(),
            services_shutdown_token.child_token(),
        );

        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let st_command_sender = Arc::downgrade(&command_sender);
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let (split_tunnel, split_tunnel_join_handle) = nym_split_tunnel::SplitTunnel::spawn(
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            route_handler.inner_handle(),
            services_shutdown_token.child_token(),
            move |st_error_cause| {
                tracing::info!("Split tunnel error: {st_error_cause:?}");

                let Some(st_command_sender) = st_command_sender.upgrade() else {
                    return;
                };

                let error_state_reason = match st_error_cause {
                    nym_split_tunnel::SplitTunnelErrorCause::Other => ErrorStateReason::SplitTunnel,
                    #[cfg(target_os = "macos")]
                    nym_split_tunnel::SplitTunnelErrorCause::NeedFullDiskPermissions => {
                        ErrorStateReason::NeedFullDiskPermissions
                    }
                    #[cfg(target_os = "macos")]
                    nym_split_tunnel::SplitTunnelErrorCause::IsOffline => {
                        tracing::warn!("ST should not report offline error!");
                        ErrorStateReason::SplitTunnel
                    }
                };

                let _ = st_command_sender.send(TunnelCommand::Block(error_state_reason));
            },
        )
        .await;

        #[cfg(target_os = "linux")]
        let split_tunnel_pid_manager = nym_split_tunnel::PidManager::default();
        #[cfg(target_os = "linux")]
        let split_tunnel_config = LinuxSplitTunnelConfiguration {
            excluded_cgroup2: split_tunnel_pid_manager.excluded_cgroup(),
            net_cls: split_tunnel_pid_manager.net_cls_classid(),
        };

        let (gateway_provider, gateway_provider_handle) = GatewayProvider::new(
            gateway_cache_handle.clone(),
            nym_vpn_api_client.clone(),
            tunnel_settings.clone(),
            wireguard_keys_db,
            state_machine_shutdown_token.child_token(),
        );

        let (file_updater, file_updater_handle) =
            nym_file_updater::FileUpdater::new().map_err(Error::CreateFileUpdater)?;
        tokio::spawn(file_updater.run(services_shutdown_token.child_token()));

        let state_machine_handle = TunnelStateMachine::spawn(
            command_receiver,
            event_sender,
            nym_config,
            tunnel_settings,
            tunnel_constants,
            account_command_tx.clone(),
            account_state_rx.clone(),
            bandwidth_command_tx.clone(),
            nym_vpn_api_client,
            statistics_event_sender.clone(),
            topology_service.clone(),
            connectivity_handle,
            discovery_refresher_command_tx,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            split_tunnel.clone(),
            gateway_provider.clone(),
            #[cfg(target_os = "linux")]
            split_tunnel_config,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_handler,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            parameters.tun_provider,
            file_updater_handle,
            parameters.user_agent.clone(),
            state_machine_shutdown_token.child_token(),
        )
        .await
        .map_err(Error::StateMachine)?;

        Ok(Self {
            paths,
            network_tx,
            user_agent: parameters.user_agent,
            vpn_command_rx,
            tunnel_event_tx,
            log_file_remover_handle,
            account_command_tx,
            account_state_rx,
            target_state: TargetState::Unsecured,
            tunnel_state,
            tunnel_settings_update_timer: Box::pin(Fuse::terminated()),
            state_machine_handle: Some(state_machine_handle),
            account_controller_handle,
            statistics_controller_handle,
            bandwidth_controller_handle,
            topology_service_join_handle,
            topology_service_handle: topology_service,
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
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            split_tunnel,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            split_tunnel_join_handle,
            gateway_provider,
            gateway_provider_handle,
            #[cfg(target_os = "linux")]
            split_tunnel_pid_manager,
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
                    self.handle_network_change(event).await;
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
            let mut fused_state_machine_handle = pin!(fused_state_machine_handle);

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

        if let Err(e) = self.bandwidth_controller_handle.await {
            tracing::error!("Failed to join on bandwidth controller handle: {e}");
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

        if let Err(e) = self.topology_service_join_handle.await {
            tracing::error!("Failed to join on statistics controller handle: {e}");
        }

        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            if let Err(e) = self.split_tunnel_join_handle.await {
                tracing::error!("Failed to join on split tunnel handle: {e}");
            }
        }

        if let Err(e) = self.gateway_provider_handle.await {
            tracing::error!("Failed to join on gateway provider handle: {e}");
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

    async fn reconnect_tunnel(&mut self) -> bool {
        match self.target_state {
            TargetState::Secured => {
                // Flush any settings update that is still pending behind the
                // throttle timer so the upcoming Connect uses the latest config.
                // Otherwise a reconnect can race ahead of the throttled
                // SetTunnelSettings and re-run with stale settings
                if !self.tunnel_settings_update_timer.is_terminated() {
                    self.tunnel_settings_update_timer.set(Fuse::terminated());
                    self.update_tunnel_settings();
                }
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

            // When VPN connects, if SOCKS5 is already enabled, warn that it may not work
            // because SOCKS5 might be using a different gateway than VPN
            if matches!(new_state, TunnelState::Connected { .. })
                && self.socks5_service.is_enabled()
            {
                tracing::warn!(
                    "VPN connected while SOCKS5 proxy was already active. SOCKS5 may be using a different gateway than VPN, which can cause connection failures. Consider disabling and re-enabling SOCKS5 to use the VPN's gateway."
                );
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

    async fn handle_network_change(&mut self, new_network: Box<Network>) {
        tracing::info!("Network environment updated");
        let _ = self.network_tx.send_replace(new_network.clone());

        // Update gateway cache and topology cache for new environment
        crate::cache_refresh::update_caches_for_network(
            &new_network,
            &self.gateway_cache_handle,
            &self.topology_service_handle,
            &self.user_agent,
        )
        .await;
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
            VpnServiceCommand::SetEnableAdBlocking(tx, enable_ad_blocking) => {
                self.handle_set_enable_ad_blocking(enable_ad_blocking).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetFrontingMode(tx, fronting_mode) => {
                self.handle_set_fronting_mode(fronting_mode).await;
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
            VpnServiceCommand::SetMixnetTrafficConfig(tx, mixnet_traffic_config) => {
                let res = self
                    .handle_set_mixnet_traffic_config(mixnet_traffic_config)
                    .await;
                let _ = tx.send(res);
            }
            VpnServiceCommand::SetGatewaySelectionAlgorithm(tx, _gateway_selection_algorithm) => {
                // self.handle_set_gateway_selection_algorithm(gateway_selection_algorithm)
                //     .await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetEnableGeoLocation(tx, enable_geo_location) => {
                let res = self
                    .handle_set_enable_geo_location(enable_geo_location)
                    .await;
                let _ = tx.send(res);
            }
            VpnServiceCommand::SetEnableGatewayIndependence(tx, enable_gateway_independence) => {
                self.handle_set_enable_gateway_independence(enable_gateway_independence)
                    .await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetGatewayIndependenceNotifications(tx, enable_notifications) => {
                self.handle_set_enable_gateway_independence_notifications(enable_notifications)
                    .await;
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
                let result = self.handle_get_default_dns();
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
            #[cfg(any(target_os = "android", target_os = "ios"))]
            VpnServiceCommand::RegisterAccount(tx, request) => {
                let _ = tx.send(self.handle_register_account(request).await);
            }
            VpnServiceCommand::CreateAccount(tx, ()) => {
                let _ = tx.send(self.handle_create_account().await);
            }
            VpnServiceCommand::StoreAccount(tx, account) => {
                let _ = tx.send(self.handle_store_account(account).await);
            }
            VpnServiceCommand::GetStoredMnemonic(tx, ()) => {
                let _ = tx.send(self.handle_get_stored_mnemonic().await);
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
            VpnServiceCommand::GetAccountMode(tx, ()) => {
                let _ = tx.send(self.handle_get_account_mode().await);
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
            VpnServiceCommand::GetCanonicalAccountIdentity(tx, ()) => {
                let _ = tx.send(self.handle_get_canonical_account_identity().await);
            }
            VpnServiceCommand::GetAccountLinks(tx, locale) => {
                let _ = tx.send(self.handle_get_account_links(locale).await);
            }
            VpnServiceCommand::GetAccountState(tx, ()) => {
                let _ = tx.send(self.handle_get_account_state().await);
            }
            VpnServiceCommand::RefreshAccountState(tx, force) => {
                self.handle_refresh_account_state(force).await;
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
            VpnServiceCommand::GetAccountSummary(tx, ()) => {
                let _ = tx.send(self.handle_get_account_summary().await);
            }
            VpnServiceCommand::HandleSubscriptionPayment(tx, ()) => {
                let _ = tx.send(self.handle_subscription_payment().await);
            }
            VpnServiceCommand::GetDeeplink(tx, params) => {
                let _ = tx.send(self.handle_get_deeplink(params).await);
            }
            VpnServiceCommand::GetAutologinDeeplink(tx, params) => {
                let _ = tx.send(self.handle_get_autologin_deeplink(params).await);
            }
            VpnServiceCommand::DeeplinkStoreAccount(tx, deeplink_callback_url) => {
                let _ = tx.send(
                    self.handle_deeplink_store_account(deeplink_callback_url)
                        .await,
                );
            }
            VpnServiceCommand::GetLogPath(tx, ()) => {
                let _ = tx.send(self.paths.log_path.clone());
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
            VpnServiceCommand::RunDiagnostic(tx, params) => {
                let _ = tx.send(self.handle_run_diagnostic(params).await);
            }
            VpnServiceCommand::RegisterDiagnostic(tx, params) => {
                let _ = tx.send(Box::pin(self.handle_register_diagnostic(params)).await);
            }
            VpnServiceCommand::GetTentativeGateways(tx, _) => {
                let _ = tx.send(self.handle_get_tentative_gateways().await);
            }
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            VpnServiceCommand::IsSplitTunnelSupported(tx, ()) => {
                let is_available = self.handle_is_split_tunnel_supported().await;
                let _ = tx.send(is_available);
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            VpnServiceCommand::SetEnableSplitTunnel(tx, enabled) => {
                self.handle_set_enable_split_tunnel(enabled).await;
                let _ = tx.send(());
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            VpnServiceCommand::AddSplitTunnelApp(tx, app) => {
                self.handle_add_split_tunnel_app(app).await;
                let _ = tx.send(());
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            VpnServiceCommand::RemoveSplitTunnelApp(tx, app) => {
                self.handle_remove_split_tunnel_app(app).await;
                let _ = tx.send(());
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            VpnServiceCommand::ClearSplitTunnelApps(tx, ()) => {
                self.handle_clear_split_tunnel_apps().await;
                let _ = tx.send(());
            }
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            VpnServiceCommand::GetSplitTunnelExcludedProcesses(tx, ()) => {
                let excluded_processes = self.handle_get_split_tunnel_excluded_processes().await;
                let _ = tx.send(excluded_processes);
            }
            #[cfg(target_os = "linux")]
            VpnServiceCommand::AddSplitTunnelProcess(tx, pid) => {
                self.handle_add_split_tunnel_process(pid);
                let _ = tx.send(());
            }
            #[cfg(target_os = "linux")]
            VpnServiceCommand::RemoveSplitTunnelProcess(tx, pid) => {
                self.handle_remove_split_tunnel_process(pid);
                let _ = tx.send(());
            }
            #[cfg(target_os = "linux")]
            VpnServiceCommand::ClearSplitTunnelProcesses(tx, ()) => {
                self.handle_clear_split_tunnel_processes();
                let _ = tx.send(());
            }
            #[cfg(target_os = "macos")]
            VpnServiceCommand::NeedFullDiskPermissions(tx, ()) => {
                let has_fda = nym_split_tunnel::has_full_disk_access();
                let _ = tx.send(!has_fda);
            }
            VpnServiceCommand::SetGeoExclusionEnabled(tx, enabled) => {
                self.handle_set_geo_exclusion_enabled(enabled).await;
                let _ = tx.send(());
            }
            VpnServiceCommand::SetGeoExclusionListenPort(tx, listen_port) => {
                let _ = tx.send(self.handle_set_geo_exclusion_listen_port(listen_port).await);
            }
            VpnServiceCommand::SetGeoExclusionExcludedCountries(tx, excluded_countries) => {
                let result = self
                    .handle_set_geo_exclusion_excluded_countries(excluded_countries)
                    .await;
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

    async fn handle_set_enable_ad_blocking(&mut self, enable_ad_blocking: bool) {
        self.config_manager
            .set_enable_ad_blocking(enable_ad_blocking)
            .await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_fronting_mode(&mut self, fronting_mode: nym_vpn_lib_types::FrontingMode) {
        self.config_manager.set_fronting_mode(fronting_mode).await;
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

    async fn handle_set_mixnet_traffic_config(
        &mut self,
        mixnet_traffic_config: MixnetTrafficConfig,
    ) -> Result<(), String> {
        self.config_manager
            .set_mixnet_traffic_config(mixnet_traffic_config)
            .await
            .map_err(|err| err.to_string())?;
        self.update_tunnel_settings_with_throttle();
        Ok(())
    }

    async fn _handle_set_gateway_selection_algorithm(
        &mut self,
        gateway_selection_algorithm: GatewaySelectionAlgorithm,
    ) {
        self.config_manager
            ._set_gateway_selection_algorithm(gateway_selection_algorithm)
            .await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_enable_geo_location(
        &mut self,
        enable_geo_location: bool,
    ) -> Result<(), String> {
        self.config_manager
            .set_enable_geo_location(enable_geo_location)
            .await
            .map_err(|err| err.to_string())?;
        self.update_tunnel_settings_with_throttle();
        Ok(())
    }

    async fn handle_set_enable_gateway_independence(&mut self, enable_gateway_independence: bool) {
        self.config_manager
            .set_enable_gateway_independence(enable_gateway_independence)
            .await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_enable_gateway_independence_notifications(
        &mut self,
        enable_notifications: bool,
    ) {
        self.config_manager
            .set_enable_gateway_independence_notifications(enable_notifications)
            .await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_network(&self, network: String) -> Result<(), SetNetworkError> {
        let mut global_config = GlobalConfig::read_from_config_dir(&self.paths.config_dir)
            .await
            .map_err(|source| SetNetworkError::ReadConfig {
                source: source.into(),
            })?;

        let network_selected = NetworkEnvironments::try_from(network.as_str())
            .map_err(|_err| SetNetworkError::NetworkNotFound(network.to_owned()))?;
        global_config.network_name = network_selected.to_string();

        global_config
            .write_to_config_dir(&self.paths.config_dir)
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

    fn handle_get_default_dns(&self) -> Vec<IpAddr> {
        // todo: return protocols too!
        let mut seen = HashSet::with_capacity(DEFAULT_DNS_SERVERS_CONFIG.len());
        DEFAULT_DNS_SERVERS_CONFIG
            .iter()
            .map(|v| v.ip)
            .filter(|ip| seen.insert(*ip))
            .collect()
    }

    async fn handle_list_gateways(
        &self,
        options: ListGatewaysOptions,
        completion_tx: oneshot::Sender<Result<Vec<Gateway>, ListGatewaysError>>,
    ) {
        let gateway_client = self.gateway_cache_handle.clone();
        let gw_type = options.gw_type;
        let mut filters = GatewayFilters::default();
        let blacklisted = self.gateway_provider.blacklisted_gateways();
        if !blacklisted.is_empty().unwrap_or(true) {
            filters.add(GatewayFilter::NotBlacklisted(blacklisted));
        }
        let filters = nym_gateway_directory::LookupGatewayFilters {
            gw_type: nym_gateway_directory::GatewayType::from(gw_type),
            filters,
        };

        tokio::spawn(async move {
            // todo: pass options.user_agent with request
            let result = gateway_client
                .lookup_filtered_gateways(filters)
                .await
                .map_err(|source| ListGatewaysError::GetGateways { gw_type, source })
                .map(|gateways| gateways.into_iter().map(Gateway::from).collect::<Vec<_>>());

            completion_tx.send(result).ok();
        });
    }

    async fn handle_list_filtered_gateways(
        &self,
        filters: LookupGatewayFilters,
        completion_tx: oneshot::Sender<Result<Vec<Gateway>, ListGatewaysError>>,
    ) {
        let gateway_client = self.gateway_cache_handle.clone();
        let gw_type = filters.gw_type;
        let mut filters: nym_gateway_directory::LookupGatewayFilters = filters.into();
        let blacklisted = self.gateway_provider.blacklisted_gateways();
        if !blacklisted.is_empty().unwrap_or(true) {
            filters
                .filters
                .add(GatewayFilter::NotBlacklisted(blacklisted));
        }

        tokio::spawn(async move {
            let result = gateway_client
                .lookup_filtered_gateways(filters)
                .await
                .map_err(|source| ListGatewaysError::GetFilteredGateways { gw_type, source })
                .map(|gateways| gateways.into_iter().map(Gateway::from).collect::<Vec<_>>());

            completion_tx.send(result).ok();
        });
    }

    async fn handle_enable_socks5(
        &mut self,
        enable_socks5_request: EnableSocks5Request,
    ) -> Result<(), Socks5Error> {
        tracing::info!("Enabling SOCKS5 client: {:?}", enable_socks5_request);

        // Get gateways from VPN API with SOCKS5 probe data
        // This includes all VPN gateways (Wg type) with SOCKS5 scores, not just MixnetExit
        let exit_gateways: nym_gateway_directory::GatewayList = self
            .gateway_cache_handle
            .lookup_nymnodes_for_socks5()
            .await
            .map_err(|e| {
                Socks5Error::InvalidConfig(format!(
                    "Failed to lookup gateways with SOCKS5 data: {}",
                    e
                ))
            })?;

        // Filter for gateways that support SOCKS5 (exit-capable with probe data)
        let exit_gateways = gateway_directory::GatewayList::new(
            None, // Mixed types (Wg gateways from VPN API)
            exit_gateways
                .into_iter()
                .filter(|gateway| {
                    // Must be exit-capable and have SOCKS5 probe data
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
            ExitPoint::Address { address } => NodeIdentity::from(*address.gateway().inner()),
            ExitPoint::Gateway { identity } => NodeIdentity::from(*identity.inner()),
            ExitPoint::Random => {
                // Random exit point: Always do random selection, ignoring VPN's exit gateway.
                // This preserves anonymity through rotation - using VPN's exit gateway would
                // always route through the same gateway, reducing anonymity benefits.
                // Note: Entry gateway still uses VPN's entry gateway (for firewall compatibility),
                // but exit gateway (Network Requester) rotates for anonymity.
                tracing::debug!("Selecting random SOCKS5 exit gateway (for rotation/anonymity)");

                let exit_point: nym_gateway_directory::ExitPoint = exit_point.clone().into();

                let exit_filters = if self.config_manager.config().residential_exit {
                    GatewayFilters::from(&[GatewayFilter::Residential, GatewayFilter::Exit])
                } else {
                    GatewayFilters::default()
                };

                let selected_gateway = exit_gateways
                    .find_best_socks5_gateway(&exit_point, &exit_filters)
                    .map_err(|e| {
                        Socks5Error::InvalidConfig(format!(
                            "Failed to select random SOCKS5 exit gateway: {e}"
                        ))
                    })?;

                tracing::info!(
                    "Selected random SOCKS5 exit gateway: {}, location: {}",
                    selected_gateway.identity(),
                    selected_gateway
                        .two_letter_iso_country_code()
                        .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
                );

                selected_gateway.identity()
            }
            ExitPoint::Country { .. } | ExitPoint::Region { .. } => {
                // For location-based exit points, check if VPN is connected first
                // If connected, use VPN's actual gateway to avoid firewall routing issues
                // (but only if it supports SOCKS5 - otherwise fall back to location-based selection)
                let tunnel_state = self.tunnel_state.read().await.clone();

                let selected_identity = if let TunnelState::Connected { connection_data } =
                    tunnel_state
                {
                    // VPN is connected - try to use its actual exit gateway
                    let vpn_gateway_id = &connection_data.exit_gateway.id;
                    tracing::info!(
                        "VPN is connected to exit gateway {}, checking if it supports SOCKS5",
                        vpn_gateway_id
                    );

                    // Validate that VPN's gateway supports SOCKS5 and has nr_address
                    match NodeIdentity::from_base58_string(vpn_gateway_id) {
                        Ok(vpn_gateway_identity) => {
                            // Look up the gateway directly (VPN uses Wg type, but gateway might also support MixnetExit)
                            let gateway_full = self
                                .gateway_cache_handle
                                .lookup_nymnode_by_identity(vpn_gateway_identity)
                                .await
                                .ok();

                            if let Some(gateway_full) = gateway_full {
                                // Check if gateway supports SOCKS5
                                // Prefer VPN API's socks5 data when available (more accurate),
                                // otherwise fall back to checking nr_address and can_connect
                                let supports_socks5 = gateway_full
                                    .last_probe
                                    .as_ref()
                                    .and_then(|probe| probe.outcome.socks5.as_ref())
                                    .map(|socks5| {
                                        // Use VPN API's SOCKS5 data - check if it has a valid score
                                        // (score being Some indicates it was probed and works)
                                        socks5.score.is_some()
                                    })
                                    .unwrap_or_else(|| {
                                        // Fallback: check nr_address and can_connect (for gateways without VPN API data yet)
                                        gateway_full.nr_address.is_some()
                                            && gateway_full
                                                .last_probe
                                                .as_ref()
                                                .and_then(|probe| probe.outcome.as_exit.as_ref())
                                                .map(|exit_point| exit_point.can_connect)
                                                .unwrap_or(false)
                                    });

                                if supports_socks5 {
                                    // Gateway supports SOCKS5 - use it directly even if not in filtered MixnetExit list
                                    // (VPN uses Wg gateways, but they may also support MixnetExit/SOCKS5)
                                    tracing::info!(
                                        "Using VPN's exit gateway {} for SOCKS5 (same gateway, firewall rules should allow connection)",
                                        vpn_gateway_id
                                    );
                                    // Use VPN's gateway identity - skip selection
                                    Some(vpn_gateway_identity)
                                } else {
                                    tracing::debug!(
                                        "VPN's exit gateway {} does not support SOCKS5 (no nr_address or cannot connect as exit), selecting different gateway",
                                        vpn_gateway_id
                                    );
                                    None
                                }
                            } else {
                                tracing::debug!(
                                    "VPN's exit gateway {} not found in cache, selecting different gateway",
                                    vpn_gateway_id
                                );
                                None
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse VPN's exit gateway identity {}: {}. Selecting new gateway.",
                                vpn_gateway_id,
                                e
                            );
                            None
                        }
                    }
                } else {
                    None
                };

                // Use VPN's gateway if available, otherwise do selection
                if let Some(gateway_identity) = selected_identity {
                    gateway_identity
                } else {
                    // VPN not connected or gateway doesn't support SOCKS5 - do selection
                    tracing::debug!("Selecting SOCKS5 exit node for exit point: {exit_point:?}",);

                    // Convert to gateway_directory types for lookup
                    let exit_point: nym_gateway_directory::ExitPoint = exit_point.clone().into();

                    let exit_filters = if self.config_manager.config().residential_exit {
                        GatewayFilters::from(&[GatewayFilter::Residential, GatewayFilter::Exit])
                    } else {
                        GatewayFilters::default()
                    };

                    let selected_gateway = exit_gateways
                        .find_best_socks5_gateway(&exit_point, &exit_filters)
                        .map_err(|e| {
                            Socks5Error::InvalidConfig(format!(
                                "Failed to select SOCKS5 exit gateway: {e}"
                            ))
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
            }
        };

        // Verify the selected gateway supports SOCKS5 (has Network Requester address)
        // Note: We don't use this NR address directly - we use random selection for privacy
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

        // Verify gateway has Network Requester support (required for SOCKS5)
        if gateway.nr_address.is_none() {
            return Err(Socks5Error::GatewayNotSupported);
        }

        let request_timeout = socks5_request_timeout();
        let idle_timeout = socks5_idle_timeout();

        // Enable Network Requester rotation for privacy - rotates every 15 minutes
        // Rotation only occurs when WireGuard VPN is connected and there are no active SOCKS5 connections
        // For privacy, start with random Network Requester (None) instead of using exit gateway's NR
        // This avoids correlation between VPN traffic and SOCKS5 traffic
        let network_requester_rotation_interval = Some(Duration::from_secs(15 * 60)); // 15 minutes
        let gateway_cache_handle = Some(self.gateway_cache_handle.clone());

        // Get current VPN exit gateway identity to exclude during random selection for privacy
        let vpn_exit_gateway_identity = {
            let tunnel_state = self.tunnel_state.read().await;
            if let TunnelState::Connected {
                ref connection_data,
            } = *tunnel_state
            {
                Some(connection_data.exit_gateway.id.clone())
            } else {
                None
            }
        };

        // Get network details from current network environment to ensure SOCKS5 uses correct network
        // Clone immediately to avoid holding watch::Ref across await (not Send)
        let network_details = Some(self.network_tx.borrow().nym_network_details().clone());

        tracing::info!(
            "Starting SOCKS5 with random Network Requester selection (excluding VPN exit gateway for privacy)"
        );

        self.socks5_service
            .enable(Socks5EnableConfig {
                data_dir: self.paths.network_data_dir.clone(),
                socks5_listen_address: enable_socks5_request.socks5_settings.listen_address,
                http_rpc_proxy_listen_address: enable_socks5_request
                    .http_rpc_settings
                    .listen_address,
                network_requester_address: None, // Start with random selection for privacy
                network_requester_rotation_interval,
                gateway_cache_handle,
                request_timeout,
                idle_timeout,
                network_details,
                vpn_exit_gateway_identity, // Exclude VPN exit gateway during random selection
            })
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

    #[cfg(any(target_os = "android", target_os = "ios"))]
    async fn handle_register_account(
        &self,
        request: RegisterAccountRequest,
    ) -> Result<RegisterAccountResponse, AccountCommandError> {
        let stored_account = self
            .account_command_tx
            .get_stored_account()
            .await?
            .ok_or(AccountCommandError::NoAccountStored)?;

        self.account_command_tx
            .register_account(
                stored_account,
                nym_vpn_api_client::types::Platform::from(request),
            )
            .await
    }

    async fn handle_create_account(&self) -> Result<(), AccountCommandError> {
        self.account_command_tx.create_account_command().await
    }

    async fn handle_store_account(
        &mut self,
        store_request: StoreAccountRequest,
    ) -> Result<(), AccountCommandError> {
        let account = StorableAccount::try_from(store_request)
            .map_err(|err| AccountCommandError::InvalidSecret(err.to_string()))?;
        self.account_command_tx.store_account(account).await
    }

    async fn handle_get_stored_mnemonic(&mut self) -> Result<String, AccountCommandError> {
        let stored_account = self
            .account_command_tx
            .get_stored_account()
            .await?
            .ok_or(AccountCommandError::NoAccountStored)?;

        Ok(stored_account.mnemonic.to_string())
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

        let data_dir = self.paths.network_data_dir.clone();
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

    async fn handle_get_canonical_account_identity(
        &self,
    ) -> Result<Option<String>, AccountCommandError> {
        self.account_command_tx.get_canonical_account_id().await
    }

    async fn handle_get_account_mode(&self) -> Option<StoredAccountMode> {
        self.account_command_tx
            .get_account_mode()
            .await
            .ok()
            .flatten()
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

    async fn handle_refresh_account_state(&self, force: bool) {
        let _ = self.account_command_tx.refresh_account_state(force).await;
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

    async fn handle_get_account_summary(
        &self,
    ) -> Result<Option<VpnAccountSummary>, AccountCommandError> {
        self.account_command_tx.get_account_summary().await
    }

    async fn handle_subscription_payment(&self) -> Result<(), AccountCommandError> {
        if !self.handle_is_account_stored().await {
            return Err(AccountCommandError::NoAccountStored);
        }
        self.account_command_tx.refresh_account_state(true).await
    }

    async fn handle_get_deeplink(
        &self,
        params: GetDeeplinkParams,
    ) -> Result<String, AccountCommandError> {
        let account_management = self
            .network_tx
            .borrow()
            .nym_vpn_network
            .account_management
            .clone()
            .ok_or(AccountCommandError::DeeplinkError(
                "No account management data is available at this time".to_string(),
            ))?;

        let base_url = match params.kind {
            DeeplinkKind::Privy | DeeplinkKind::PrivyLink => {
                let url = match params.client {
                    DeeplinkClient::Mobile => account_management.privy_mobile_url(&params.locale),
                    DeeplinkClient::Desktop => account_management.privy_desktop_url(&params.locale),
                    DeeplinkClient::Web => account_management.privy_web_url(&params.locale),
                };
                url.ok_or(AccountCommandError::DeeplinkError(
                    "The privy path could not be determined".to_string(),
                ))?
            }
            DeeplinkKind::CreateAccount => account_management.pricing_url(&params.locale).ok_or(
                AccountCommandError::DeeplinkError(
                    "The create account path could not be determined".to_string(),
                ),
            )?,
            _ => {
                return Err(AccountCommandError::DeeplinkError(
                    "Invalid deeplink kind".to_string(),
                ));
            }
        };

        self.account_command_tx
            .get_deeplink(params.kind, params.name, base_url)
            .await
    }

    async fn handle_get_autologin_deeplink(
        &self,
        params: GetDeeplinkParams,
    ) -> Result<AutologinResponse, AccountCommandError> {
        let account_management = self
            .network_tx
            .borrow()
            .nym_vpn_network
            .account_management
            .clone()
            .ok_or(AccountCommandError::DeeplinkError(
                "No account management data is available at this time".to_string(),
            ))?;

        let opt_url = match params.client {
            DeeplinkClient::Mobile => account_management.autologin_mobile_url(&params.locale),
            DeeplinkClient::Desktop => account_management.autologin_desktop_url(&params.locale),
            DeeplinkClient::Web => account_management.autologin_web_url(&params.locale),
        };

        let base_url = opt_url.ok_or(AccountCommandError::DeeplinkError(
            "The autologin path could not be determined".to_string(),
        ))?;

        self.account_command_tx
            .get_autologin_deeplink(params.kind, params.name, base_url)
            .await
    }

    async fn handle_deeplink_store_account(
        &self,
        deeplink_url: String,
    ) -> Result<(), AccountCommandError> {
        let deeplink_mnemonic = self
            .account_command_tx
            .derive_deeplink_mnemonic(deeplink_url)
            .await?;

        let account = StorableAccount {
            mnemonic: deeplink_mnemonic.mnemonic,
            mode: StoredAccountMode::Privy,
        };

        match deeplink_mnemonic.kind {
            DeeplinkKind::Privy | DeeplinkKind::CreateAccount => {
                self.account_command_tx.store_account(account).await
            }
            DeeplinkKind::PrivyLink => self.account_command_tx.link_account(account).await,
            _ => Err(AccountCommandError::DeeplinkError(
                "Invalid deeplink kind".to_string(),
            )),
        }
    }

    async fn handle_delete_log_file(&self) {
        if let Some(remove_log_file_handle) = self.log_file_remover_handle.as_ref() {
            remove_log_file_handle.remove_log_file();
        }
    }

    async fn handle_is_sentry_enabled(&self) -> bool {
        GlobalConfig::read_from_config_dir(&self.paths.config_dir)
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
        let mut config = GlobalConfig::read_from_config_dir(&self.paths.config_dir)
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
        config
            .write_to_config_dir(&self.paths.config_dir)
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

    async fn handle_run_diagnostic(&self, params: DiagnosticRunParams) -> DiagnosticReport {
        let network = *self.network_tx.borrow().clone();
        let report = DiagnosticHandler::run(network, params).await;
        match serde_json::to_string_pretty(&report) {
            Ok(report_log) => tracing::info!("{report_log}"),
            Err(e) => tracing::error!("Error serializing report :{e}"),
        }
        report
    }

    async fn handle_register_diagnostic(
        &self,
        mut params: DiagnosticRegisterParams,
    ) -> RegistrationReport {
        if !(*self.tunnel_state.read().await == TunnelState::Disconnected
            && self.account_state_rx.get_state() == AccountControllerState::ReadyToConnect)
        {
            return RegistrationReport::from_err(
                "Must be disconnected and ready to connect to run registration diagnostic",
            );
        }
        let network = *self.network_tx.borrow().clone();
        if params.storage_path.is_none() {
            params.storage_path = Some(self.paths.network_data_dir.clone());
        }
        let report = Box::pin(DiagnosticHandler::register(network, params)).await;
        match serde_json::to_string_pretty(&report) {
            Ok(report_log) => tracing::info!("{report_log}"),
            Err(e) => tracing::error!("Error serializing report :{e}"),
        }
        report
    }

    async fn handle_get_tentative_gateways(&self) -> TentativeGateways {
        self.gateway_provider.tentative_gateways().await
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn handle_is_split_tunnel_supported(&mut self) -> bool {
        #[cfg(target_os = "linux")]
        {
            self.split_tunnel_pid_manager.is_supported()
        }

        #[cfg(not(target_os = "linux"))]
        {
            true
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    async fn handle_set_enable_split_tunnel(&mut self, enabled: bool) {
        self.config_manager.set_enable_split_tunnel(enabled).await;
        self.update_tunnel_settings_with_throttle();
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    async fn handle_add_split_tunnel_app(&mut self, app: SplitApp) {
        self.config_manager.add_split_tunnel_app(app).await;
        self.update_tunnel_settings_with_throttle();
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    async fn handle_remove_split_tunnel_app(&mut self, app: SplitApp) {
        self.config_manager.remove_split_tunnel_app(app).await;
        self.update_tunnel_settings_with_throttle();
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    async fn handle_clear_split_tunnel_apps(&mut self) {
        self.config_manager.clear_split_tunnel_apps().await;
        self.update_tunnel_settings_with_throttle();
    }

    #[cfg(target_os = "macos")]
    async fn handle_get_split_tunnel_excluded_processes(
        &mut self,
    ) -> SplitTunnelExcludedProcessList {
        let result = self.split_tunnel.get_excluded_processes().await;

        result.unwrap_or_default()
    }

    #[cfg(target_os = "linux")]
    async fn handle_get_split_tunnel_excluded_processes(
        &mut self,
    ) -> SplitTunnelExcludedProcessList {
        let processes = self
            .split_tunnel_pid_manager
            .list()
            .unwrap_or_default()
            .into_iter()
            .map(|pid| SplitTunnelExcludedProcess {
                pid,
                // disregarded fields
                exec_path: PathBuf::default(),
                responsible_exec_path: PathBuf::default(),
            })
            .collect::<Vec<_>>();
        SplitTunnelExcludedProcessList { processes }
    }

    #[cfg(target_os = "linux")]
    fn handle_add_split_tunnel_process(&mut self, pid: i32) {
        if let Err(err) = self.split_tunnel_pid_manager.add(pid) {
            trace_err_chain!(err, "failed to add process to exclusions");
        }
    }

    #[cfg(target_os = "linux")]
    fn handle_remove_split_tunnel_process(&mut self, pid: i32) {
        if let Err(err) = self.split_tunnel_pid_manager.remove(pid) {
            trace_err_chain!(err, "failed to remove process from exclusions");
        }
    }

    #[cfg(target_os = "linux")]
    fn handle_clear_split_tunnel_processes(&mut self) {
        if let Err(err) = self.split_tunnel_pid_manager.clear() {
            trace_err_chain!(err, "failed to remove all processes from exclusions");
        }
    }

    async fn handle_set_geo_exclusion_enabled(&mut self, enabled: bool) {
        self.config_manager.set_geo_exclusion_enabled(enabled).await;
        self.update_tunnel_settings_with_throttle();
    }

    async fn handle_set_geo_exclusion_listen_port(
        &mut self,
        listen_port: u16,
    ) -> Result<(), GeoExclusionConfigError> {
        self.config_manager
            .set_geo_exclusion_listen_port(listen_port)
            .await?;
        self.update_tunnel_settings_with_throttle();
        Ok(())
    }

    async fn handle_set_geo_exclusion_excluded_countries(
        &mut self,
        excluded_countries: Vec<String>,
    ) -> Result<(), GeoExclusionConfigError> {
        self.config_manager
            .set_geo_exclusion_excluded_countries(excluded_countries)
            .await?;
        self.update_tunnel_settings_with_throttle();
        Ok(())
    }
}
