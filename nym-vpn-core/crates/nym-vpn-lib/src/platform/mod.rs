// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! The Uniffi generated bindings for the Nym VPN library. The API is designed to be used by
//! frontends to interact with the Nym VPN library. The API is designed to be platform agnostic and
//! should work on any platform that supports the Uniffi FFI bindings.
//!
//! Usage:
//!
//! 1. Initialize the environment: `initEnvironment(..)` or `initFallbackMainnetEnvironment`.
//!
//!     This is required to set the network environment details.
//!
//! 2. Initialise the library: `configureLib(..)`.
//!
//!     This sets up the logger and starts the account controller that runs in the background and
//!     manages the account state.
//!
//! 3. At this point we can interact with the vpn-api and the account controller to do things like:
//!
//!     - Get gateway countries: `getGatewayCountries(..)`.
//!     - Store the account mnemonic: `storeAccountMnemonic(..)`.
//!     - Get the account state: `getAccountState()`.
//!     - Get system messages: `getSystemMessages()`.
//!     - Get account links: `getAccountLinks(..)`.
//!     - ...
//!
//! 3. Start the VPN: `startVPN(..)`.
//!
//!     This will:
//!
//!     1. Check if the account is ready to connect.
//!     2. Request zknym credentials if needed.
//!     3. Start the VPN state machine.
//!
//! 4. Stop the VPN: `stopVPN()`.
//!
//!     This will stop the VPN state machine.
//!
//! 5. Shutdown the library: `shutdown()`.
//!
//!     This will stop the account controller and clean up any resources, including make sure there
//!     are no open DB connections.

#[cfg(target_os = "android")]
pub mod android;
pub(crate) mod error;
pub mod helpers;
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod swift;

mod account;
mod environment;
mod state_machine;

use std::{
    env,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use account::AccountControllerHandle;
use lazy_static::lazy_static;
use nym_gateway_directory::{NodeIdentity, Recipient};
use nym_wg_go::PublicKey;
use time::OffsetDateTime;
use tokio::{runtime::Runtime, sync::Mutex};

use state_machine::StateMachineHandle;

use self::error::VpnError;
#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::ios::OSTunProvider;
use crate::{
    gateway_directory::GatewayClient,
    uniffi_custom_impls::{
        AccountLinks, AccountStateSummary, BandwidthStatus, ConnectionStatus, EntryPoint,
        ExitPoint, GatewayMinPerformance, GatewayType, Location, NetworkEnvironment, SystemMessage,
        TunStatus, UserAgent,
    },
};
use nym_vpn_lib_types::{
    ActionAfterDisconnect as CoreActionAfterDisconnect, BandwidthEvent as CoreBandwidthEvent,
    ConnectionData as CoreConnectionData, ConnectionEvent as CoreConnectionEvent,
    ConnectionStatisticsEvent as CoreConnectionStatisticsEvent,
    ErrorStateReason as CoreErrorStateReason, MixnetConnectionData as CoreMixnetConnectionData,
    MixnetEvent as CoreMixnetEvent, SphinxPacketRates as CoreSphinxPacketRates,
    TunnelConnectionData as CoreTunnelConnectionData, TunnelEvent as CoreTunnelEvent,
    TunnelState as CoreTunnelState, WireguardConnectionData as CoreWireguardConnectionData,
    WireguardNode as CoreWireguardNode,
};

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref STATE_MACHINE_HANDLE: Mutex<Option<StateMachineHandle>> = Mutex::new(None);
    static ref ACCOUNT_CONTROLLER_HANDLE: Mutex<Option<AccountControllerHandle>> = Mutex::new(None);
    static ref NETWORK_ENVIRONMENT: Mutex<Option<nym_vpn_network_config::Network>> =
        Mutex::new(None);
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

/// Setup the library with the given data directory and optionally enable credential mode.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn configureLib(data_dir: String, credential_mode: Option<bool>) -> Result<(), VpnError> {
    RUNTIME.block_on(configure_lib(data_dir, credential_mode))
}

async fn configure_lib(data_dir: String, credential_mode: Option<bool>) -> Result<(), VpnError> {
    let network = environment::current_environment_details().await?;
    account::init_account_controller(PathBuf::from(data_dir), credential_mode, network).await
}

fn init_logger(path: Option<PathBuf>) {
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    tracing::info!("Setting log level: {log_level}, path?: {path:?}");
    #[cfg(target_os = "ios")]
    swift::init_logs(log_level, path);
    #[cfg(target_os = "android")]
    android::init_logs(log_level);
}

/// Additional extra function for when only only want to set the logger without initializing the
/// library. Thus it's only needed when `configureLib` is not used.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn initLogger(path: Option<PathBuf>) {
    init_logger(path);
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

/// Get the liset of countries that have gateways available of the given type.
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

/// Start the VPN by first establishing that the account is ready to connect, including requesting
/// zknym credentials, and then starting the VPN state machine.
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
    let timeout = Duration::from_secs(120);
    account::wait_for_account_ready_to_connect(enable_credentials_mode, timeout).await?;

    // Once we have established that the account is ready, we can start the state machine.
    state_machine::init_state_machine(config, network_env, enable_credentials_mode).await
}

async fn is_credential_mode_enabled(credential_mode: Option<bool>) -> Result<bool, VpnError> {
    match credential_mode {
        Some(enable_credentials_mode) => Ok(enable_credentials_mode),
        None => environment::get_feature_flag_credential_mode().await,
    }
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
    RUNTIME.block_on(account::stop_account_controller())
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

#[derive(uniffi::Enum)]
pub enum TunnelEvent {
    NewState(TunnelState),
    MixnetState(MixnetEvent),
}

impl From<CoreTunnelEvent> for TunnelEvent {
    fn from(value: CoreTunnelEvent) -> Self {
        match value {
            CoreTunnelEvent::NewState(new_state) => Self::NewState(TunnelState::from(new_state)),
            CoreTunnelEvent::MixnetState(event) => Self::MixnetState(MixnetEvent::from(event)),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum TunnelState {
    Disconnected,
    Connecting {
        connection_data: Option<ConnectionData>,
    },
    Connected {
        connection_data: ConnectionData,
    },
    Disconnecting {
        after_disconnect: ActionAfterDisconnect,
    },
    Error(ErrorStateReason),
    Offline {
        reconnect: bool,
    },
}

impl From<CoreTunnelState> for TunnelState {
    fn from(value: CoreTunnelState) -> Self {
        match value {
            CoreTunnelState::Connected { connection_data } => TunnelState::Connected {
                connection_data: ConnectionData::from(connection_data),
            },
            CoreTunnelState::Connecting { connection_data } => TunnelState::Connecting {
                connection_data: connection_data.map(ConnectionData::from),
            },
            CoreTunnelState::Disconnecting { after_disconnect } => TunnelState::Disconnecting {
                after_disconnect: ActionAfterDisconnect::from(after_disconnect),
            },
            CoreTunnelState::Disconnected => TunnelState::Disconnected,
            CoreTunnelState::Error(reason) => TunnelState::Error(ErrorStateReason::from(reason)),
            CoreTunnelState::Offline { reconnect } => TunnelState::Offline { reconnect },
        }
    }
}

#[derive(uniffi::Enum)]
pub enum MixnetEvent {
    Bandwidth(BandwidthEvent),
    Connection(ConnectionEvent),
    ConnectionStatistics(ConnectionStatisticsEvent),
}

impl From<CoreMixnetEvent> for MixnetEvent {
    fn from(value: CoreMixnetEvent) -> Self {
        match value {
            CoreMixnetEvent::Bandwidth(event) => Self::Bandwidth(BandwidthEvent::from(event)),
            CoreMixnetEvent::Connection(event) => Self::Connection(ConnectionEvent::from(event)),
            CoreMixnetEvent::ConnectionStatistics(event) => {
                Self::ConnectionStatistics(ConnectionStatisticsEvent::from(event))
            }
        }
    }
}

#[derive(uniffi::Record)]
pub struct ConnectionStatisticsEvent {
    pub rates: SphinxPacketRates,
}

impl From<CoreConnectionStatisticsEvent> for ConnectionStatisticsEvent {
    fn from(value: CoreConnectionStatisticsEvent) -> Self {
        Self {
            rates: SphinxPacketRates::from(value.rates),
        }
    }
}

impl From<CoreSphinxPacketRates> for SphinxPacketRates {
    fn from(value: CoreSphinxPacketRates) -> Self {
        Self {
            real_packets_sent: value.real_packets_sent,
            real_packets_sent_size: value.real_packets_sent_size,
            cover_packets_sent: value.cover_packets_sent,
            cover_packets_sent_size: value.cover_packets_sent_size,
            real_packets_received: value.real_packets_received,
            real_packets_received_size: value.real_packets_received_size,
            cover_packets_received: value.cover_packets_received,
            cover_packets_received_size: value.cover_packets_received_size,
            total_acks_received: value.total_acks_received,
            total_acks_received_size: value.total_acks_received_size,
            real_acks_received: value.real_acks_received,
            real_acks_received_size: value.real_acks_received_size,
            cover_acks_received: value.cover_acks_received,
            cover_acks_received_size: value.cover_acks_received_size,
            real_packets_queued: value.real_packets_queued,
            retransmissions_queued: value.retransmissions_queued,
            reply_surbs_queued: value.reply_surbs_queued,
            additional_reply_surbs_queued: value.additional_reply_surbs_queued,
        }
    }
}

#[derive(uniffi::Record)]
pub struct SphinxPacketRates {
    pub real_packets_sent: f64,
    pub real_packets_sent_size: f64,
    pub cover_packets_sent: f64,
    pub cover_packets_sent_size: f64,

    pub real_packets_received: f64,
    pub real_packets_received_size: f64,
    pub cover_packets_received: f64,
    pub cover_packets_received_size: f64,

    pub total_acks_received: f64,
    pub total_acks_received_size: f64,
    pub real_acks_received: f64,
    pub real_acks_received_size: f64,
    pub cover_acks_received: f64,
    pub cover_acks_received_size: f64,

    pub real_packets_queued: f64,
    pub retransmissions_queued: f64,
    pub reply_surbs_queued: f64,
    pub additional_reply_surbs_queued: f64,
}

#[derive(uniffi::Enum)]
pub enum BandwidthEvent {
    NoBandwidth,
    RemainingBandwidth(i64),
}

impl From<CoreBandwidthEvent> for BandwidthEvent {
    fn from(value: CoreBandwidthEvent) -> Self {
        match value {
            CoreBandwidthEvent::NoBandwidth => BandwidthEvent::NoBandwidth,
            CoreBandwidthEvent::RemainingBandwidth(value) => {
                BandwidthEvent::RemainingBandwidth(value)
            }
        }
    }
}

#[derive(uniffi::Enum)]
pub enum ConnectionEvent {
    EntryGatewayDown,
    ExitGatewayDownIpv4,
    ExitGatewayDownIpv6,
    ExitGatewayRoutingErrorIpv4,
    ExitGatewayRoutingErrorIpv6,
    ConnectedIpv4,
    ConnectedIpv6,
}

impl From<CoreConnectionEvent> for ConnectionEvent {
    fn from(value: CoreConnectionEvent) -> Self {
        match value {
            CoreConnectionEvent::EntryGatewayDown => Self::EntryGatewayDown,
            CoreConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            CoreConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            CoreConnectionEvent::ExitGatewayRoutingErrorIpv4 => Self::ExitGatewayRoutingErrorIpv4,
            CoreConnectionEvent::ExitGatewayRoutingErrorIpv6 => Self::ExitGatewayRoutingErrorIpv6,
            CoreConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            CoreConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum ActionAfterDisconnect {
    Nothing,
    Reconnect,
    Offline,
    Error,
}

impl From<CoreActionAfterDisconnect> for ActionAfterDisconnect {
    fn from(value: CoreActionAfterDisconnect) -> Self {
        match value {
            CoreActionAfterDisconnect::Nothing => Self::Nothing,
            CoreActionAfterDisconnect::Reconnect => Self::Reconnect,
            CoreActionAfterDisconnect::Error => Self::Error,
            CoreActionAfterDisconnect::Offline => Self::Offline,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum ErrorStateReason {
    Firewall,
    Routing,
    Dns,
    TunDevice,
    TunnelProvider,
    SameEntryAndExitGateway,
    InvalidEntryGatewayCountry,
    InvalidExitGatewayCountry,
    BadBandwidthIncrease,
    DuplicateTunFd,
    Internal,
}

impl From<CoreErrorStateReason> for ErrorStateReason {
    fn from(value: CoreErrorStateReason) -> Self {
        match value {
            CoreErrorStateReason::Firewall => Self::Firewall,
            CoreErrorStateReason::Routing => Self::Routing,
            CoreErrorStateReason::Dns => Self::Dns,
            CoreErrorStateReason::TunDevice => Self::TunDevice,
            CoreErrorStateReason::TunnelProvider => Self::TunnelProvider,
            CoreErrorStateReason::SameEntryAndExitGateway => Self::SameEntryAndExitGateway,
            CoreErrorStateReason::InvalidEntryGatewayCountry => Self::InvalidEntryGatewayCountry,
            CoreErrorStateReason::InvalidExitGatewayCountry => Self::InvalidExitGatewayCountry,
            CoreErrorStateReason::BadBandwidthIncrease => Self::BadBandwidthIncrease,
            CoreErrorStateReason::DuplicateTunFd => Self::DuplicateTunFd,
            CoreErrorStateReason::Internal => Self::Internal,
        }
    }
}

#[derive(uniffi::Record)]
pub struct ConnectionData {
    pub entry_gateway: Box<NodeIdentity>,
    pub exit_gateway: Box<NodeIdentity>,
    pub connected_at: Option<OffsetDateTime>,
    pub tunnel: TunnelConnectionData,
}

impl From<CoreConnectionData> for ConnectionData {
    fn from(value: CoreConnectionData) -> Self {
        Self {
            entry_gateway: value.entry_gateway,
            exit_gateway: value.exit_gateway,
            connected_at: value.connected_at,
            tunnel: TunnelConnectionData::from(value.tunnel),
        }
    }
}

impl From<CoreTunnelConnectionData> for TunnelConnectionData {
    fn from(value: CoreTunnelConnectionData) -> Self {
        match value {
            CoreTunnelConnectionData::Mixnet(data) => {
                TunnelConnectionData::Mixnet(MixnetConnectionData::from(data))
            }
            CoreTunnelConnectionData::Wireguard(data) => {
                TunnelConnectionData::Wireguard(WireguardConnectionData::from(data))
            }
        }
    }
}

impl From<CoreMixnetConnectionData> for MixnetConnectionData {
    fn from(value: CoreMixnetConnectionData) -> Self {
        Self {
            nym_address: value.nym_address,
            exit_ipr: value.exit_ipr,
            ipv4: value.ipv4,
            ipv6: value.ipv6,
        }
    }
}

impl From<CoreWireguardConnectionData> for WireguardConnectionData {
    fn from(value: CoreWireguardConnectionData) -> Self {
        Self {
            entry: WireguardNode::from(value.entry),
            exit: WireguardNode::from(value.exit),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum TunnelConnectionData {
    Mixnet(MixnetConnectionData),
    Wireguard(WireguardConnectionData),
}

#[derive(uniffi::Record)]
pub struct MixnetConnectionData {
    pub nym_address: Box<Recipient>,
    pub exit_ipr: Box<Recipient>,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
}

#[derive(uniffi::Record)]
pub struct WireguardConnectionData {
    pub entry: WireguardNode,
    pub exit: WireguardNode,
}

#[derive(uniffi::Record)]
pub struct WireguardNode {
    pub endpoint: SocketAddr,
    pub public_key: Box<PublicKey>,
    pub private_ipv4: Ipv4Addr,
    pub private_ipv6: Ipv6Addr,
}

impl From<CoreWireguardNode> for WireguardNode {
    fn from(value: CoreWireguardNode) -> Self {
        Self {
            endpoint: value.endpoint,
            public_key: value.public_key,
            private_ipv4: value.private_ipv4,
            private_ipv6: value.private_ipv6,
        }
    }
}

#[uniffi::export(with_foreign)]
pub trait TunnelStatusListener: Send + Sync {
    fn on_event(&self, event: TunnelEvent);
}

impl From<&CoreTunnelState> for TunStatus {
    fn from(value: &CoreTunnelState) -> Self {
        // TODO: this cannot be accurate so we must switch frontends to use TunnelState instead! But for now that will do.
        match value {
            CoreTunnelState::Connecting { .. } => Self::EstablishingConnection,
            CoreTunnelState::Connected { .. } => Self::Up,
            CoreTunnelState::Disconnecting { .. } => Self::Disconnecting,
            CoreTunnelState::Disconnected => Self::Down,
            CoreTunnelState::Error(_) => Self::Down,
            CoreTunnelState::Offline { .. } => Self::Down,
        }
    }
}

impl From<CoreBandwidthEvent> for BandwidthStatus {
    fn from(value: CoreBandwidthEvent) -> Self {
        match value {
            CoreBandwidthEvent::NoBandwidth => Self::NoBandwidth,
            CoreBandwidthEvent::RemainingBandwidth(bandwidth) => {
                Self::RemainingBandwidth { bandwidth }
            }
        }
    }
}

impl From<CoreConnectionEvent> for ConnectionStatus {
    fn from(value: CoreConnectionEvent) -> Self {
        match value {
            CoreConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            CoreConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
            CoreConnectionEvent::EntryGatewayDown => Self::EntryGatewayDown,
            CoreConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            CoreConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            CoreConnectionEvent::ExitGatewayRoutingErrorIpv4 => Self::ExitGatewayRoutingErrorIpv4,
            CoreConnectionEvent::ExitGatewayRoutingErrorIpv6 => Self::ExitGatewayRoutingErrorIpv6,
        }
    }
}
