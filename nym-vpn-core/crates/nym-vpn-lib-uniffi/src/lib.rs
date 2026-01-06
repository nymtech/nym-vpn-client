// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! The Uniffi generated bindings for the Nym VPN library. The API is designed to be used by
//! frontends to interact with the Nym VPN library. The API is designed to be platform-agnostic and
//! should work on any platform that supports the Uniffi FFI bindings.
//!
//! Usage:
//!
//! 1. Initialize the environment: `initEnvironment(..)` or `initFallbackMainnetEnvironment`.
//!
//!    This is required to set the network environment details.
//!
//! 2. Initialize the library: `configureLib(..)`.
//!
//!    This sets up the logger and starts the account controller that runs in the background and
//!    manages the account state. It also starts the statistics controller running in the background.
//!
//! 3. At this point we can interact with the vpn-api and the account controller to do things like:
//!
//!    - Get gateway countries: `getGatewayCountries(..)`.
//!    - Store the account mnemonic: `storeAccountMnemonic(..)`.
//!    - Get the account state: `getAccountState()`.
//!    - Get system messages: `getSystemMessages()`.
//!    - Get account links: `getAccountLinks(..)`.
//!    - ...
//!
//! 3. Start the VPN: `startVPN(..)`.
//!
//!    This will:
//!
//!    1. Check if the account is ready to connect.
//!    2. Request zknym credentials if needed.
//!    3. Start the VPN state machine.
//!
//! 4. Stop the VPN: `stopVPN()`.
//!
//!    This will stop the VPN state machine.
//!
//! 5. Shutdown the library: `shutdown()`.
//!
//!    This will stop the account controller and clean up any resources, including make sure there
//!    are no open DB connections.

uniffi::setup_scaffolding!();

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
pub mod ios;

pub(crate) mod error;
pub mod helpers;

mod account;
mod environment;
mod gateway_cache;
mod offline_monitor;
mod sentry_monitoring;
mod state_machine;
mod stats;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod tunnel_provider;

use std::{
    env,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::Arc,
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use lazy_static::lazy_static;
use nym_platform_metadata::SysInfo;
use nym_vpn_store::keys::wireguard::WireguardKeysDb;
use sentry::ClientInitGuard;
use tokio::{runtime::Runtime, sync::Mutex};

use nym_vpn_lib_types::{
    AccountControllerState, EntryPoint, ExitPoint, Gateway, GatewayType, Network,
    NetworkCompatibility, ParsedAccountLinks, PrivyDerivationMessage, RegisterAccountResponse,
    StoreAccountRequest, SystemMessage, TunnelEvent, UserAgent, NymVpnUsage
};

use account::AccountControllerHandle;
use error::VpnError;
use gateway_cache::UniffiGatewayCacheHandle;
use offline_monitor::OfflineMonitorHandle;
use state_machine::StateMachineHandle;
use stats::StatisticsControllerHandle;
#[cfg(target_os = "android")]
use tunnel_provider::android::{AndroidConnectivityMonitor, AndroidTunProvider};
#[cfg(target_os = "ios")]
use tunnel_provider::ios::OSTunProvider;

uniffi::use_remote_type!(nym_vpn_lib_types::IpAddr);
uniffi::use_remote_type!(nym_vpn_lib_types::Ipv4Addr);
uniffi::use_remote_type!(nym_vpn_lib_types::Ipv6Addr);
uniffi::use_remote_type!(nym_vpn_lib_types::IpNetwork);
uniffi::use_remote_type!(nym_vpn_lib_types::Ipv4Network);
uniffi::use_remote_type!(nym_vpn_lib_types::Ipv6Network);
uniffi::use_remote_type!(nym_vpn_lib_types::PathBuf);

// todo: refactor this, stop using statics
lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref OFFLINE_MONITOR_HANDLE: Mutex<Option<OfflineMonitorHandle>> = Mutex::new(None);
    static ref STATE_MACHINE_HANDLE: Mutex<Option<StateMachineHandle>> = Mutex::new(None);
    static ref ACCOUNT_CONTROLLER_HANDLE: Mutex<Option<AccountControllerHandle>> = Mutex::new(None);
    static ref WIREGUARD_KEYS_DB: Mutex<Option<WireguardKeysDb>> = Mutex::new(None);
    static ref STATISTICS_CONTROLLER_HANDLE: Mutex<Option<StatisticsControllerHandle>> =
        Mutex::new(None);
    static ref NETWORK_ENVIRONMENT: Mutex<Option<nym_vpn_network_config::Network>> =
        Mutex::new(None);
    static ref GATEWAY_CACHE: Mutex<Option<UniffiGatewayCacheHandle>> = Mutex::new(None);
    static ref SENTRY_CLIENT: Mutex<Option<ClientInitGuard>> = Mutex::new(None);
}

/// Fetches the network environment details from the network name and initializes the environment,
/// including exporting to the environment
#[allow(non_snake_case)]
#[uniffi::export]
pub fn initEnvironment(cache_dir: String, network_name: &str) -> Result<(), VpnError> {
    RUNTIME.block_on(environment::init_environment(cache_dir, network_name))
}

/// Async variant of initEnvironment. Fetches the network environment details from the network name
/// and initializes the environment, including exporting to the environment
#[allow(non_snake_case)]
#[uniffi::export]
pub async fn initEnvironmentAsync(data_dir: String, network_name: &str) -> Result<(), VpnError> {
    environment::init_environment(data_dir, network_name).await
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
pub fn currentEnvironment() -> Result<Network, VpnError> {
    RUNTIME.block_on(environment::current_environment())
}

/// Setup the library with the given data directory and optionally enable credential mode.
/// On iOS: use this function to configure the lib in network extension.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn configureLib(config: NymVpnLibConfig) -> Result<(), VpnError> {
    RUNTIME.block_on(configure_lib(config))
}

/// Setup the library for the main process on iOS.
#[allow(non_snake_case)]
#[uniffi::export]
#[cfg(target_os = "ios")]
pub fn configureLibForMainProcess(user_agent: UserAgent) -> Result<(), VpnError> {
    RUNTIME.block_on(configure_lib_for_main_process(user_agent))
}

async fn configure_lib(config: NymVpnLibConfig) -> Result<(), VpnError> {
    let network = environment::current_environment_details().await?;
    let os = SysInfo::new();
    println!("OS information: {os}");
    if config.sentry_monitoring {
        let mut guard = SENTRY_CLIENT.lock().await;
        *guard = sentry_monitoring::init();
    }
    offline_monitor::init_offline_monitor(
        #[cfg(target_os = "android")]
        config.connectivity_monitor,
    )
    .await?;
    stats::init_statistics_controller(
        PathBuf::from(config.data_dir.clone()),
        network.clone(),
        config.statistics_enabled,
    )
    .await?;
    Box::pin(account::init_account_controller(
        PathBuf::from(config.data_dir),
        network,
    ))
    .await?;

    let connectivity_handle = offline_monitor::get_connectivity_handle().await?;
    gateway_cache::init_gateway_cache(config.user_agent, connectivity_handle).await?;

    Ok(())
}

#[cfg(target_os = "ios")]
async fn configure_lib_for_main_process(user_agent: UserAgent) -> Result<(), VpnError> {
    offline_monitor::init_offline_monitor().await?;

    let connectivity_handle = offline_monitor::get_connectivity_handle().await?;
    gateway_cache::init_gateway_cache(user_agent, connectivity_handle).await?;

    Ok(())
}

async fn init_logger(
    path: Option<PathBuf>,
    debug_level: Option<String>,
    sentry_monitoring: bool,
) -> Result<(), VpnError> {
    let default_log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    let log_level = debug_level.unwrap_or(default_log_level);
    tracing::info!("Setting log level: {log_level}, path?: {path:?}");
    let os = SysInfo::new();
    tracing::info!("OS information: {}", os);
    if sentry_monitoring {
        let mut guard = SENTRY_CLIENT.lock().await;
        *guard = sentry_monitoring::init();
    }

    #[cfg(target_os = "ios")]
    {
        ios::init_logs(log_level, path, sentry_monitoring)
    }
    #[cfg(target_os = "android")]
    {
        android::init_logs(log_level)
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        Ok(())
    }
}

/// Additional extra function for when only want to set the logger without initializing the
/// library. Thus, it's only needed when `configureLib` is not used.
#[allow(non_snake_case)]
#[uniffi::export]
pub async fn initLogger(
    path: Option<PathBuf>,
    debug_level: Option<String>,
    sentry_monitoring: bool,
) -> Result<(), VpnError> {
    init_logger(path, debug_level, sentry_monitoring).await
}

/// Returns the system messages for the current network environment
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getSystemMessages() -> Result<Vec<SystemMessage>, VpnError> {
    RUNTIME.block_on(environment::get_system_messages())
}

/// Returns the oldest client versions that are compatible with the
/// network environment. (environment must be initialized first)
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getNetworkCompatibilityVersions() -> Result<Option<NetworkCompatibility>, VpnError> {
    RUNTIME.block_on(environment::get_network_compatibility())
}

/// Returns the account links for the current network environment
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getAccountLinks(locale: &str) -> Result<ParsedAccountLinks, VpnError> {
    RUNTIME.block_on(environment::get_account_links(locale))
}

/// Returns the account links for the current network environment.
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getAccountLinksRaw(
    account_store_path: &str,
    locale: &str,
) -> Result<ParsedAccountLinks, VpnError> {
    RUNTIME.block_on(environment::get_account_links_raw(
        account_store_path,
        locale,
    ))
}

/// Import the account mnemonic
#[allow(non_snake_case)]
#[uniffi::export]
pub fn login(request: StoreAccountRequest) -> Result<(), VpnError> {
    RUNTIME.block_on(account::login(&request))
}

/// Generate the account mnemonic locally and store it.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn createAccount() -> Result<(), VpnError> {
    RUNTIME.block_on(account::create_account())
}

/// Register the stored account.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn registerAccount(args: AccountRegistrationArgs) -> Result<RegisterAccountResponse, VpnError> {
    RUNTIME.block_on(account::register_account(args))
}

/// Store the account mnemonic
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn loginRaw(request: StoreAccountRequest, path: String) -> Result<(), VpnError> {
    RUNTIME.block_on(account::raw::login_raw(&request, &path))
}

/// Generate the account mnemonic locally and store it.
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn createAccountRaw(path: String) -> Result<(), VpnError> {
    RUNTIME.block_on(account::raw::create_account_raw(&path))
}

/// Load the account mnemonic stored locally and register it.
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn registerAccountRaw(path: String) -> Result<RegisterAccountResponse, VpnError> {
    RUNTIME.block_on(account::raw::register_account_raw(&path))
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

/// Read and return the mnemonic, if there's one stored.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getStoredMnemonicRaw(path: String) -> Result<String, VpnError> {
    RUNTIME.block_on(account::raw::get_stored_mnemonic_raw(&path))
}

/// Read and return the mnemonic, if there's one stored.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getStoredMnemonic() -> Result<String, VpnError> {
    RUNTIME.block_on(account::get_stored_mnemonic())
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

/// Force a rotation of the wireguard keys
#[allow(non_snake_case)]
#[uniffi::export]
pub fn rotateKeys() -> Result<(), VpnError> {
    RUNTIME.block_on(account::rotate_keys())
}

/// Force a rotation of the wireguard keys
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn rotateKeysRaw(path: String) -> Result<(), VpnError> {
    RUNTIME.block_on(account::raw::rotate_keys_raw(&path))
}

/// Get the device identity
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getDeviceIdentity() -> Result<String, VpnError> {
    RUNTIME.block_on(account::get_device_id())
}

/// Get the account identity
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getAccountIdentity() -> Result<String, VpnError> {
    RUNTIME.block_on(get_account_id())
}

/// Get the device identity
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getDeviceIdentityRaw(path: String) -> Result<String, VpnError> {
    RUNTIME.block_on(account::raw::get_device_id_raw(&path))
}

/// Get the account identity
/// This is a version that can be called when the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getAccountIdentityRaw(path: String) -> Result<String, VpnError> {
    RUNTIME.block_on(account::raw::get_account_id_raw(&path))
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
pub fn getAccountState() -> Result<AccountControllerState, VpnError> {
    RUNTIME.block_on(account::get_account_state())
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn getAccountUsage() -> Result<Vec<NymVpnUsage>, VpnError> {
    RUNTIME.block_on(account::get_account_usage())
}

async fn get_account_id() -> Result<String, VpnError> {
    account::get_account_id()
        .await?
        .ok_or(VpnError::NoAccountStored)
}

/// Get the list of gateways available of the given type.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getGateways(gw_type: GatewayType) -> Result<Vec<Gateway>, VpnError> {
    RUNTIME.block_on(get_gateways(gw_type))
}

/// Get the message to be signed using the Privy signing API.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getPrivyDerivationMessage() -> PrivyDerivationMessage {
    PrivyDerivationMessage {
        message: nym_vpn_lib::login::privy::message_to_sign(),
    }
}

async fn get_gateways(gw_type: GatewayType) -> Result<Vec<Gateway>, VpnError> {
    gateway_cache::get_gateway_cache_handle()
        .await?
        .lookup_gateways(gw_type.into())
        .await
        .map(|gateways| {
            gateways
                .into_inner()
                .into_iter()
                .map(Gateway::from)
                .collect()
        })
        .map_err(|err| VpnError::NetworkConnectionError {
            details: err.to_string(),
        })
}

/// Start the VPN by first establishing that the account is ready to connect, including requesting
/// zknym credentials, and then starting the VPN state machine.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn startVPN(config: VPNConfig) -> Result<(), VpnError> {
    RUNTIME.block_on(start_vpn_inner(Box::new(config)))
}

async fn start_vpn_inner(config: Box<VPNConfig>) -> Result<(), VpnError> {
    log_build_info();

    // Get the network environment details. This relies on the network environment being set in
    // advance by calling initEnvironment or initFallbackMainnetEnvironment.
    let network_env = environment::current_environment_details().await?;

    let account_controller_tx = account::get_command_sender().await?;
    let account_controller_state = account::get_state_receiver().await?;
    let wireguard_key_db = account::get_wireguard_key_db().await?;

    let statistics_event_sender = stats::get_events_sender().await?;

    // Once we have established that the account is ready, we can start the state machine.
    state_machine::init_state_machine(
        config,
        Box::new(network_env),
        account_controller_tx,
        account_controller_state,
        wireguard_key_db,
        statistics_event_sender,
    )
    .await
}

fn log_build_info() {
    let build_info = nym_bin_common::bin_info_local_vergen!();
    tracing::info!(
        "{} {} ({})",
        build_info.binary_name,
        build_info.build_version,
        build_info.commit_sha
    );
}

/// Stop the VPN by stopping the VPN state machine.
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

/// Shutdown the library by stopping the account controller and cleaning up any resources.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn shutdown() -> Result<(), VpnError> {
    RUNTIME.block_on(shutdown_lib())
}

pub async fn shutdown_lib() -> Result<(), VpnError> {
    stats::stop_statistics_controller().await?;
    account::stop_account_controller().await?;
    gateway_cache::stop_gateway_cache().await
}

#[derive(uniffi::Record)]
pub struct VPNConfig {
    pub entry_gateway: EntryPoint,
    pub exit_router: ExitPoint,
    pub enable_two_hop: bool,
    pub enable_bridges: bool,
    pub residential_exit: bool,
    /// Custom DNS used when set.
    /// Leave empty to use default DNS servers.
    pub custom_dns: Vec<IpAddr>,
    #[cfg(target_os = "android")]
    pub tun_provider: Arc<dyn AndroidTunProvider>,
    #[cfg(target_os = "ios")]
    pub tun_provider: Arc<dyn OSTunProvider>,
    pub config_path: Option<PathBuf>,
    pub credential_data_path: Option<PathBuf>,
    pub tun_status_listener: Option<Arc<dyn TunnelStatusListener>>,
    pub statistics_recipient: Option<String>,
    pub user_agent: UserAgent,
}

#[uniffi::export(with_foreign)]
pub trait TunnelStatusListener: Send + Sync {
    fn on_event(&self, event: TunnelEvent);
}

#[derive(uniffi::Record)]
pub struct NymVpnLibConfig {
    pub data_dir: String,
    pub sentry_monitoring: bool,
    pub statistics_enabled: bool,
    #[cfg(target_os = "android")]
    pub connectivity_monitor: Arc<dyn AndroidConnectivityMonitor>,
    pub user_agent: UserAgent,
}

#[derive(uniffi::Record)]
pub struct AccountRegistrationArgs {
    #[cfg(target_os = "android")]
    pub purchase_token: String,
}

impl TryFrom<AccountRegistrationArgs> for nym_vpn_api_client::types::Platform {
    type Error = VpnError;

    fn try_from(_value: AccountRegistrationArgs) -> Result<Self, Self::Error> {
        #[cfg(target_os = "ios")]
        return Ok(nym_vpn_api_client::types::Platform::Apple);
        #[cfg(target_os = "android")]
        return Ok(nym_vpn_api_client::types::Platform::Android {
            purchase_token: _value.purchase_token,
        });
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Err(VpnError::InternalError {
            details: "only iOS and Android supported for now".to_string(),
        })
    }
}
