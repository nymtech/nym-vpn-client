// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, path::PathBuf, sync::Arc, time::Instant};

use bip39::Mnemonic;
use nym_vpn_network_config::{
    FeatureFlags, Network, NetworkCompatibility, NymNetwork, NymVpnNetwork, ParsedAccountLinks,
    SystemMessages,
};
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::{
    sync::{broadcast, mpsc, oneshot, watch},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use nym_vpn_account_controller::{
    AccountCommand, AccountController, AccountControllerCommander, AccountStateSummary,
    AvailableTicketbooks, SharedAccountState,
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
    MixnetClientConfig, Recipient, UserAgent,
};
use nym_vpn_lib_types::{TunnelEvent, TunnelState, TunnelType};
use zeroize::Zeroizing;

use super::{
    config::{NetworkEnvironments, NymVpnServiceConfig, DEFAULT_CONFIG_FILE},
    error::{AccountError, Error, Result, SetNetworkError, VpnServiceDeleteLogFileError},
    VpnServiceConnectError, VpnServiceDisconnectError,
};
use crate::config::GlobalConfigFile;
use crate::logging::LogPath;

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
    Connect(
        oneshot::Sender<Result<(), VpnServiceConnectError>>,
        ConnectArgs,
    ),
    Disconnect(oneshot::Sender<Result<(), VpnServiceDisconnectError>>, ()),
    GetTunnelState(oneshot::Sender<TunnelState>, ()),
    SubscribeToTunnelState(oneshot::Sender<watch::Receiver<TunnelState>>, ()),
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
    GetLogPath(oneshot::Sender<Option<LogPath>>, ()),
    DeleteLogFile(
        oneshot::Sender<Result<(), VpnServiceDeleteLogFileError>>,
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

#[derive(Clone, Debug)]
pub struct VpnServiceInfo {
    pub version: String,
    pub build_timestamp: Option<OffsetDateTime>,
    pub triple: String,
    pub platform: String,
    pub git_commit: String,
    pub nym_network: NymNetwork,
    pub nym_vpn_network: NymVpnNetwork,
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
    account_command_tx: AccountControllerCommander,

    // Path to the main config file
    config_file: PathBuf,

    // Path to the data directory
    data_dir: PathBuf,

    // If log to file is enabled, path to the log directory and log filename
    log_path: Option<LogPath>,

    // Storage backend
    storage: Arc<tokio::sync::Mutex<S>>,

    // Last known tunnel state wrapped in a `watch::Sender` that can be used to track tunnel state individually.
    tunnel_state: watch::Sender<TunnelState>,

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
    pub fn spawn(
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        tunnel_event_tx: broadcast::Sender<TunnelEvent>,
        file_logging_event_tx: mpsc::Sender<()>,
        shutdown_token: CancellationToken,
        network_env: Network,
        user_agent: UserAgent,
        log_path: Option<LogPath>,
    ) -> JoinHandle<()> {
        tracing::trace!("Starting VPN service");
        tokio::spawn(async {
            match NymVpnService::new(
                vpn_command_rx,
                tunnel_event_tx,
                file_logging_event_tx,
                shutdown_token,
                network_env,
                user_agent,
                log_path,
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

    pub async fn new(
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        tunnel_event_tx: broadcast::Sender<TunnelEvent>,
        file_logging_event_tx: mpsc::Sender<()>,
        shutdown_token: CancellationToken,
        network_env: Network,
        user_agent: UserAgent,
        log_path: Option<LogPath>,
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

        let statistics_recipient = network_env
            .system_configuration
            .and_then(|config| config.statistics_recipient);

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
        let nyxd_url = network_env.nyxd_url();
        let api_url = network_env.api_url();
        let gateway_config = gateway_directory::Config {
            nyxd_url,
            api_url,
            nym_vpn_api_url: Some(network_env.vpn_api_url()),
            min_gateway_performance: None,
            mix_score_thresholds: None,
            wg_score_thresholds: None,
        };
        let nym_config = NymConfig {
            config_path: Some(config_dir),
            data_path: Some(data_dir.clone()),
            gateway_config,
            network_env: network_env.clone(),
        };

        let state_machine_handle = TunnelStateMachine::spawn(
            command_receiver,
            event_sender,
            nym_config,
            tunnel_settings,
            account_command_tx.clone(),
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
            data_dir,
            log_path,
            storage,
            tunnel_state: watch::Sender::new(TunnelState::Disconnected),
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
    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                Some(command) = self.vpn_command_rx.recv() => {
                    tracing::debug!("Received command: {command}");
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

        if let Err(e) = self.state_machine_handle.await {
            tracing::error!("Failed to join on state machine handle: {}", e);
        }

        tracing::info!("Exiting vpn service run loop");

        Ok(())
    }

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
            VpnServiceCommand::GetLogPath(tx, ()) => {
                let _ = tx.send(self.log_path.clone());
            }
            VpnServiceCommand::DeleteLogFile(tx, ()) => {
                let result = self.handle_delete_log_file().await;
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
        self.network_env.network_compatibility.clone()
    }

    async fn handle_get_feature_flags(&self) -> Option<FeatureFlags> {
        self.network_env.feature_flags.clone()
    }

    async fn handle_store_account(
        &mut self,
        account: Zeroizing<String>,
    ) -> Result<(), AccountError> {
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
        if *self.tunnel_state.borrow() != TunnelState::Disconnected {
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

    async fn handle_reset_device_identity(
        &mut self,
        seed: Option<[u8; 32]>,
    ) -> Result<(), AccountError> {
        if *self.tunnel_state.borrow() != TunnelState::Disconnected {
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
}
