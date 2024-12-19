// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
pub mod android;
pub(crate) mod error;
pub mod helpers;
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod swift;

mod account;
mod environment;

use std::{env, path::PathBuf, sync::Arc, time::Duration};

use account::AccountControllerHandle;
use lazy_static::lazy_static;
use log::*;
use nym_vpn_network_config::Network;
use tokio::{
    runtime::Runtime,
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use nym_gateway_directory::Config as GatewayDirectoryConfig;

use self::error::VpnError;
use crate::platform::account::start_account_controller_handle;
#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::ios::OSTunProvider;
use crate::{
    gateway_directory::GatewayClient,
    tunnel_state_machine::{
        BandwidthEvent, ConnectionEvent, DnsOptions, GatewayPerformanceOptions,
        MixnetTunnelOptions, NymConfig, TunnelCommand, TunnelEvent, TunnelSettings, TunnelState,
        TunnelStateMachine, TunnelType, WireguardTunnelOptions,
    },
    uniffi_custom_impls::{
        AccountLinks, AccountStateSummary, BandwidthStatus, ConnectionStatus, EntryPoint,
        ExitPoint, GatewayMinPerformance, GatewayType, Location, NetworkEnvironment, SystemMessage,
        TunStatus, UserAgent,
    },
};

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref STATE_MACHINE_HANDLE: Mutex<Option<StateMachineHandle>> = Mutex::new(None);
    static ref ACCOUNT_CONTROLLER_HANDLE: Mutex<Option<AccountControllerHandle>> = Mutex::new(None);
    static ref NETWORK_ENVIRONMENT: Mutex<Option<nym_vpn_network_config::Network>> =
        Mutex::new(None);
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn startVPN(config: VPNConfig) -> Result<(), VpnError> {
    RUNTIME.block_on(start_vpn_inner(config))
}

async fn start_vpn_inner(config: VPNConfig) -> Result<(), VpnError> {
    // Get the network environment details. This relies on the network environment being set in
    // advance by calling initEnvironment or initFallbackMainnetEnvironment.
    let network_env = environment::current_environment_details().await?;

    // Enabling credential mode will depend on the network feature flag as well as what is passed
    // in the config.
    let enable_credentials_mode = is_credential_mode_enabled(config.credential_mode).await?;

    // TODO: we do a pre-connect check here. This mirrors the logic in the daemon.
    // We want to move this check into the state machine so that it happens during the connecting
    // state instead. This would allow us more flexibility in waiting for the account to be ready
    // and handle errors in a unified manner.
    // This can take a surprisingly long time, if we need to go through all steps of registering
    // the device and requesting zknym ticketbooks.
    let timeout = Duration::from_secs(60);
    account::wait_for_account_ready_to_connect(enable_credentials_mode, timeout).await?;

    // Once we have established that the account is ready, we can start the state machine.
    init_state_machine(config, network_env, enable_credentials_mode).await
}

async fn init_state_machine(
    config: VPNConfig,
    network_env: Network,
    enable_credentials_mode: bool,
) -> Result<(), VpnError> {
    let mut guard = STATE_MACHINE_HANDLE.lock().await;

    if guard.is_none() {
        let state_machine_handle =
            start_state_machine(config, network_env, enable_credentials_mode).await?;
        state_machine_handle.send_command(TunnelCommand::Connect);
        *guard = Some(state_machine_handle);
        Ok(())
    } else {
        Err(VpnError::InvalidStateError {
            details: "State machine is already running.".to_owned(),
        })
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn stopVPN() -> Result<(), VpnError> {
    RUNTIME.block_on(stop_vpn_inner())
}

async fn stop_vpn_inner() -> Result<(), VpnError> {
    let mut guard = STATE_MACHINE_HANDLE.lock().await;

    match guard.take() {
        Some(state_machine_handle) => {
            // TODO: add timeout
            state_machine_handle.shutdown_and_wait().await;
            Ok(())
        }
        None => Err(VpnError::InvalidStateError {
            details: "State machine is not running.".to_owned(),
        }),
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn configureLib(data_dir: String, credential_mode: Option<bool>) -> Result<(), VpnError> {
    RUNTIME.block_on(reconfigure_library(data_dir, credential_mode))
}

async fn reconfigure_library(
    data_dir: String,
    credential_mode: Option<bool>,
) -> Result<(), VpnError> {
    // stop if already running
    let _ = account::stop_account_controller_handle().await;

    init_logger();
    let network = environment::current_environment_details().await?;

    start_account_controller_handle(PathBuf::from(data_dir), credential_mode, network).await
}

async fn is_credential_mode_enabled(credential_mode: Option<bool>) -> Result<bool, VpnError> {
    match credential_mode {
        Some(enable_credentials_mode) => Ok(enable_credentials_mode),
        None => environment::get_feature_flag_credential_mode().await,
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn shutdown() -> Result<(), VpnError> {
    RUNTIME.block_on(account::stop_account_controller_handle())
}

fn init_logger() {
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    info!("Setting log level: {}", log_level);
    #[cfg(target_os = "ios")]
    swift::init_logs(log_level);
    #[cfg(target_os = "android")]
    android::init_logs(log_level);
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn initLogger() {
    init_logger();
}

/// Fetches the network environment details from the network name and initializes the environment,
/// including exporting to the environment
#[allow(non_snake_case)]
#[uniffi::export]
pub fn initEnvironment(network_name: &str) -> Result<(), VpnError> {
    RUNTIME.block_on(environment::init_environment(network_name))
}

/// Async variant of initEnvironment. Fetches the network environment details from the network name
/// and initializes the environment, including exporting to the environment
#[allow(non_snake_case)]
#[uniffi::export]
pub async fn initEnvironmentAsync(network_name: &str) -> Result<(), VpnError> {
    environment::init_environment(network_name).await
}

/// Sets up mainnet defaults without making any network calls. This means no system messages or
/// account links will be available.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn initFallbackMainnetEnvironment() -> Result<(), VpnError> {
    RUNTIME.block_on(environment::init_fallback_mainnet_environment())
}

/// Returns the currently set network environment
#[allow(non_snake_case)]
#[uniffi::export]
pub fn currentEnvironment() -> Result<NetworkEnvironment, VpnError> {
    RUNTIME.block_on(environment::current_environment())
}

/// Returns the system messages for the current network environment
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getSystemMessages() -> Result<Vec<SystemMessage>, VpnError> {
    RUNTIME.block_on(environment::get_system_messages())
}

/// Returns the account links for the current network environment
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getAccountLinks(locale: &str) -> Result<AccountLinks, VpnError> {
    RUNTIME.block_on(environment::get_account_links(locale))
}

/// Returns the account links for the current network environment.
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getAccountLinksRaw(
    account_store_path: &str,
    locale: &str,
) -> Result<AccountLinks, VpnError> {
    RUNTIME.block_on(environment::get_account_links_raw(
        account_store_path,
        locale,
    ))
}

/// Store the account mnemonic
#[allow(non_snake_case)]
#[uniffi::export]
pub fn storeAccountMnemonic(mnemonic: String) -> Result<(), VpnError> {
    RUNTIME.block_on(account::store_account_mnemonic(&mnemonic))
}

/// Store the account mnemonic
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn storeAccountMnemonicRaw(mnemonic: String, path: String) -> Result<(), VpnError> {
    RUNTIME.block_on(account::raw::store_account_mnemonic_raw(&mnemonic, &path))
}

/// Check if the account mnemonic is stored
#[allow(non_snake_case)]
#[uniffi::export]
pub fn isAccountMnemonicStored() -> Result<bool, VpnError> {
    RUNTIME.block_on(account::is_account_mnemonic_stored())
}

/// Check if the account mnemonic is stored
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn isAccountMnemonicStoredRaw(path: String) -> Result<bool, VpnError> {
    RUNTIME.block_on(account::raw::is_account_mnemonic_stored_raw(&path))
}

/// Remove the account mnemonic and all associated keys and files
#[allow(non_snake_case)]
#[uniffi::export]
pub fn forgetAccount() -> Result<(), VpnError> {
    RUNTIME.block_on(account::forget_account())
}

/// Remove the account mnemonic and all associated keys and files.
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn forgetAccountRaw(path: String) -> Result<(), VpnError> {
    RUNTIME.block_on(account::raw::forget_account_raw(&path))
}

/// Get the device identity
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getDeviceIdentity() -> Result<String, VpnError> {
    RUNTIME.block_on(account::get_device_id())
}

/// Get the device identity
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getDeviceIdentityRaw(path: String) -> Result<String, VpnError> {
    RUNTIME.block_on(account::raw::get_device_id_raw(&path))
}

/// This manually syncs the account state with the server. Normally this is done automatically, but
/// this can be used to manually trigger a sync.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn updateAccountState() -> Result<(), VpnError> {
    RUNTIME.block_on(account::update_account_state())
}

/// Get the account state
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getAccountState() -> Result<AccountStateSummary, VpnError> {
    RUNTIME.block_on(account::get_account_state())
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn getGatewayCountries(
    gw_type: GatewayType,
    user_agent: UserAgent,
    min_gateway_performance: Option<GatewayMinPerformance>,
) -> Result<Vec<Location>, VpnError> {
    RUNTIME.block_on(get_gateway_countries(
        gw_type,
        user_agent,
        min_gateway_performance,
    ))
}

async fn get_gateway_countries(
    gw_type: GatewayType,
    user_agent: UserAgent,
    min_gateway_performance: Option<GatewayMinPerformance>,
) -> Result<Vec<Location>, VpnError> {
    let network_env = environment::current_environment_details().await?;
    let api_url = network_env.api_url().ok_or(VpnError::InternalError {
        details: "API URL not found".to_string(),
    })?;
    let nym_vpn_api_url = Some(network_env.vpn_api_url());
    let min_gateway_performance = min_gateway_performance.map(|p| p.try_into()).transpose()?;
    let directory_config = nym_gateway_directory::Config {
        api_url,
        nym_vpn_api_url,
        min_gateway_performance,
    };
    GatewayClient::new(directory_config, user_agent.into())?
        .lookup_countries(gw_type.into())
        .await
        .map(|countries| countries.into_iter().map(Location::from).collect())
        .map_err(VpnError::from)
}

#[derive(uniffi::Record)]
pub struct VPNConfig {
    pub entry_gateway: EntryPoint,
    pub exit_router: ExitPoint,
    pub enable_two_hop: bool,
    #[cfg(target_os = "android")]
    pub tun_provider: Arc<dyn AndroidTunProvider>,
    #[cfg(target_os = "ios")]
    pub tun_provider: Arc<dyn OSTunProvider>,
    pub credential_data_path: Option<PathBuf>,
    pub tun_status_listener: Option<Arc<dyn TunnelStatusListener>>,
    pub credential_mode: Option<bool>,
    pub statistics_recipient: Option<String>,
    pub user_agent: UserAgent,
}

#[uniffi::export(with_foreign)]
pub trait TunnelStatusListener: Send + Sync {
    fn on_event(&self, event: TunnelEvent);
}

struct StateMachineHandle {
    state_machine_handle: JoinHandle<()>,
    event_broadcaster_handler: JoinHandle<()>,
    command_sender: mpsc::UnboundedSender<TunnelCommand>,
    shutdown_token: CancellationToken,
}

impl StateMachineHandle {
    fn send_command(&self, command: TunnelCommand) {
        if let Err(e) = self.command_sender.send(command) {
            tracing::error!("Failed to send comamnd: {}", e);
        }
    }

    async fn shutdown_and_wait(self) {
        self.shutdown_token.cancel();

        if let Err(e) = self.state_machine_handle.await {
            tracing::error!("Failed to join on state machine handle: {}", e);
        }

        if let Err(e) = self.event_broadcaster_handler.await {
            tracing::error!("Failed to join on event broadcaster handle: {}", e);
        }
    }
}

async fn start_state_machine(
    config: VPNConfig,
    network_env: Network,
    enable_credentials_mode: bool,
) -> Result<StateMachineHandle, VpnError> {
    let tunnel_type = if config.enable_two_hop {
        TunnelType::Wireguard
    } else {
        TunnelType::Mixnet
    };

    let statistics_recipient = config
        .statistics_recipient
        .map(nym_gateway_directory::Recipient::try_from_base58_string)
        .transpose()
        .inspect_err(|err| {
            tracing::error!("Failed to parse statistics recipient: {}", err);
        })
        .unwrap_or_default()
        .map(Box::new);

    let entry_point = nym_gateway_directory::EntryPoint::from(config.entry_gateway);
    let exit_point = nym_gateway_directory::ExitPoint::from(config.exit_router);

    let api_url = network_env.api_url().ok_or(VpnError::InternalError {
        details: "API URL not found".to_string(),
    })?;
    let nym_vpn_api_url = Some(network_env.vpn_api_url());

    let gateway_config = GatewayDirectoryConfig {
        api_url,
        nym_vpn_api_url,
        ..Default::default()
    };

    let nym_config = NymConfig {
        data_path: config.credential_data_path,
        gateway_config,
    };

    let tunnel_settings = TunnelSettings {
        tunnel_type,
        enable_credentials_mode,
        statistics_recipient,
        mixnet_tunnel_options: MixnetTunnelOptions::default(),
        wireguard_tunnel_options: WireguardTunnelOptions::default(),
        gateway_performance_options: GatewayPerformanceOptions::default(),
        mixnet_client_config: None,
        entry_point: Box::new(entry_point),
        exit_point: Box::new(exit_point),
        dns: DnsOptions::default(),
        user_agent: Some(config.user_agent.into()),
    };

    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let (event_sender, mut event_receiver) = mpsc::unbounded_channel();

    let state_listener = config.tun_status_listener;
    let event_broadcaster_handler = tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            if let Some(ref state_listener) = state_listener {
                (*state_listener).on_event(event);
            }
        }
    });

    let shutdown_token = CancellationToken::new();
    let state_machine_handle = TunnelStateMachine::spawn(
        command_receiver,
        event_sender,
        nym_config,
        tunnel_settings,
        #[cfg(any(target_os = "ios", target_os = "android"))]
        config.tun_provider,
        shutdown_token.child_token(),
    )
    .await?;

    Ok(StateMachineHandle {
        state_machine_handle,
        event_broadcaster_handler,
        command_sender,
        shutdown_token,
    })
}

impl From<&TunnelState> for TunStatus {
    fn from(value: &TunnelState) -> Self {
        // TODO: this cannot be accurate so we must switch frontends to use TunnelState instead! But for now that will do.
        match value {
            TunnelState::Connecting { .. } => Self::EstablishingConnection,
            TunnelState::Connected { .. } => Self::Up,
            TunnelState::Disconnecting { .. } => Self::Disconnecting,
            TunnelState::Disconnected => Self::Down,
            TunnelState::Error(_) => Self::Down,
        }
    }
}

impl From<BandwidthEvent> for BandwidthStatus {
    fn from(value: BandwidthEvent) -> Self {
        match value {
            BandwidthEvent::NoBandwidth => Self::NoBandwidth,
            BandwidthEvent::RemainingBandwidth(bandwidth) => Self::RemainingBandwidth { bandwidth },
        }
    }
}

impl From<ConnectionEvent> for ConnectionStatus {
    fn from(value: ConnectionEvent) -> Self {
        match value {
            ConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            ConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
            ConnectionEvent::EntryGatewayDown => Self::EntryGatewayDown,
            ConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            ConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            ConnectionEvent::ExitGatewayRoutingErrorIpv4 => Self::ExitGatewayRoutingErrorIpv4,
            ConnectionEvent::ExitGatewayRoutingErrorIpv6 => Self::ExitGatewayRoutingErrorIpv6,
        }
    }
}
