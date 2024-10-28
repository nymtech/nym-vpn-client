// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

#[cfg(target_os = "android")]
pub mod android;
pub(crate) mod error;
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod swift;

use std::{env, path::PathBuf, str::FromStr, sync::Arc};

use lazy_static::lazy_static;
use log::*;
use tokio::{
    runtime::Runtime,
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use url::Url;

use nym_gateway_directory::Config as GatewayDirectoryConfig;
use nym_vpn_store::mnemonic::MnemonicStorage as _;

use self::error::VpnError;
#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::ios::OSTunProvider;
use crate::{
    gateway_directory::GatewayClient,
    tunnel_state_machine::{
        BandwidthEvent, ConnectionEvent, DnsOptions, GatewayPerformanceOptions,
        MixnetTunnelOptions, NymConfig, TunnelCommand, TunnelEvent, TunnelSettings, TunnelState,
        TunnelStateMachine, TunnelType,
    },
    uniffi_custom_impls::{
        BandwidthStatus, ConnectionStatus, EntryPoint, ExitPoint, GatewayMinPerformance,
        GatewayType, Location, TunStatus, UserAgent,
    },
};

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref STATE_MACHINE_HANDLE: Mutex<Option<StateMachineHandle>> = Mutex::new(None);
    static ref ACCOUNT_CONTROLLER_HANDLE: Mutex<Option<AccountControllerHandle>> = Mutex::new(None);
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn startVPN(config: VPNConfig) -> Result<(), VpnError> {
    RUNTIME.block_on(start_vpn_inner(config))
}

async fn start_vpn_inner(config: VPNConfig) -> Result<(), VpnError> {
    let mut guard = STATE_MACHINE_HANDLE.lock().await;

    if guard.is_none() {
        let state_machine_handle = start_state_machine(config).await?;
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
pub fn startAccountController(data_dir: String) -> Result<(), VpnError> {
    RUNTIME.block_on(start_account_controller_inner(PathBuf::from(data_dir)))
}

async fn start_account_controller_inner(data_dir: PathBuf) -> Result<(), VpnError> {
    let mut guard = ACCOUNT_CONTROLLER_HANDLE.lock().await;

    if guard.is_none() {
        let account_controller_handle = start_account_controller(data_dir).await?;
        *guard = Some(account_controller_handle);
        Ok(())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Account controller is already running.".to_owned(),
        })
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn stopAccountController() -> Result<(), VpnError> {
    RUNTIME.block_on(stop_account_controller_inner())
}

async fn stop_account_controller_inner() -> Result<(), VpnError> {
    let mut guard = ACCOUNT_CONTROLLER_HANDLE.lock().await;

    match guard.take() {
        Some(account_controller_handle) => {
            account_controller_handle.shutdown_and_wait().await;
            Ok(())
        }
        None => Err(VpnError::InvalidStateError {
            details: "Account controller is not running.".to_owned(),
        }),
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn initLogger() {
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    info!("Setting log level: {}", log_level);
    #[cfg(target_os = "ios")]
    swift::init_logs(log_level);
    #[cfg(target_os = "android")]
    android::init_logs(log_level);
}

fn setup_account_storage(path: &str) -> Result<crate::storage::VpnClientOnDiskStorage, VpnError> {
    let path = PathBuf::from_str(path).map_err(|err| VpnError::InternalError {
        details: err.to_string(),
    })?;
    Ok(crate::storage::VpnClientOnDiskStorage::new(path))
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn storeAccountMnemonic(mnemonic: String, path: String) -> Result<(), VpnError> {
    RUNTIME.block_on(store_account_mnemonic(&mnemonic, &path))
}

async fn store_account_mnemonic(mnemonic: &str, path: &str) -> Result<(), VpnError> {
    let storage = setup_account_storage(path)?;

    let mnemonic = nym_vpn_store::mnemonic::Mnemonic::parse(mnemonic).map_err(|err| {
        VpnError::InternalError {
            details: err.to_string(),
        }
    })?;

    storage
        .store_mnemonic(mnemonic)
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

    Ok(())
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn isAccountMnemonicStored(path: String) -> Result<bool, VpnError> {
    RUNTIME.block_on(is_account_mnemonic_stored(&path))
}

async fn is_account_mnemonic_stored(path: &str) -> Result<bool, VpnError> {
    let storage = setup_account_storage(path)?;
    storage
        .is_mnemonic_stored()
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn removeAccountMnemonic(path: String) -> Result<bool, VpnError> {
    RUNTIME.block_on(remove_account_mnemonic(&path))
}

async fn remove_account_mnemonic(path: &str) -> Result<bool, VpnError> {
    let storage = setup_account_storage(path)?;
    storage
        .remove_mnemonic()
        .await
        .map(|_| true)
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })
}

#[allow(non_snake_case, dead_code)]
pub fn getAccountSummary(
    path: String,
    nym_vpn_api_url: Url,
    user_agent: UserAgent,
) -> Result<String, VpnError> {
    RUNTIME.block_on(get_account_summary(path, nym_vpn_api_url, user_agent))
}

async fn get_account_summary(
    _path: String,
    _nym_vpn_api_url: Url,
    _user_agent: UserAgent,
) -> Result<String, VpnError> {
    let guard = ACCOUNT_CONTROLLER_HANDLE.lock().await;

    if let Some(guard) = &*guard {
        let shared_account_state = guard.shared_state.lock().await.clone();
        Ok(shared_account_state.to_string())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Account controller is not running.".to_owned(),
        })
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn getGatewayCountries(
    api_url: Url,
    nym_vpn_api_url: Option<Url>,
    gw_type: GatewayType,
    user_agent: Option<UserAgent>,
    min_gateway_performance: Option<GatewayMinPerformance>,
) -> Result<Vec<Location>, VpnError> {
    RUNTIME.block_on(get_gateway_countries(
        api_url,
        nym_vpn_api_url,
        gw_type,
        user_agent,
        min_gateway_performance,
    ))
}

async fn get_gateway_countries(
    api_url: Url,
    nym_vpn_api_url: Option<Url>,
    gw_type: GatewayType,
    user_agent: Option<UserAgent>,
    min_gateway_performance: Option<GatewayMinPerformance>,
) -> Result<Vec<Location>, VpnError> {
    let user_agent = user_agent
        .map(nym_sdk::UserAgent::from)
        .unwrap_or_else(crate::util::construct_user_agent);
    let min_gateway_performance = min_gateway_performance.map(|p| p.try_into()).transpose()?;
    let directory_config = nym_gateway_directory::Config {
        api_url,
        nym_vpn_api_url,
        min_gateway_performance,
    };
    GatewayClient::new(directory_config, user_agent)?
        .lookup_countries(gw_type.into())
        .await
        .map(|countries| countries.into_iter().map(Location::from).collect())
        .map_err(VpnError::from)
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn getLowLatencyEntryCountry(
    api_url: Url,
    vpn_api_url: Option<Url>,
    user_agent: UserAgent,
) -> Result<Location, VpnError> {
    RUNTIME.block_on(get_low_latency_entry_country(
        api_url,
        vpn_api_url,
        Some(user_agent),
    ))
}

async fn get_low_latency_entry_country(
    api_url: Url,
    vpn_api_url: Option<Url>,
    user_agent: Option<UserAgent>,
) -> Result<Location, VpnError> {
    let config = nym_gateway_directory::Config {
        api_url,
        nym_vpn_api_url: vpn_api_url,
        min_gateway_performance: None,
    };
    let user_agent = user_agent
        .map(nym_sdk::UserAgent::from)
        .unwrap_or_else(crate::util::construct_user_agent);

    GatewayClient::new(config, user_agent)?
        .lookup_low_latency_entry_gateway()
        .await
        .map_err(VpnError::from)
        .and_then(|gateway| {
            gateway.location.ok_or(VpnError::InternalError {
                details: "gateway does not contain a two character country ISO".to_string(),
            })
        })
        .map(Location::from)
}

#[derive(uniffi::Record)]
pub struct VPNConfig {
    pub api_url: Url,
    pub vpn_api_url: Option<Url>,
    pub entry_gateway: EntryPoint,
    pub exit_router: ExitPoint,
    pub enable_two_hop: bool,
    #[cfg(target_os = "android")]
    pub tun_provider: Arc<dyn AndroidTunProvider>,
    #[cfg(target_os = "ios")]
    pub tun_provider: Arc<dyn OSTunProvider>,
    pub credential_data_path: Option<PathBuf>,
    pub tun_status_listener: Option<Arc<dyn TunnelStatusListener>>,
}

#[uniffi::export(with_foreign)]
pub trait TunnelStatusListener: Send + Sync {
    fn on_event(&self, event: TunnelEvent);
}

struct AccountControllerHandle {
    command_sender: mpsc::UnboundedSender<nym_vpn_account_controller::AccountCommand>,
    shared_state: nym_vpn_account_controller::SharedAccountState,
    handle: JoinHandle<()>,
    shutdown_token: CancellationToken,
}

impl AccountControllerHandle {
    fn send_command(&self, command: nym_vpn_account_controller::AccountCommand) {
        if let Err(e) = self.command_sender.send(command) {
            tracing::error!("Failed to send comamnd: {}", e);
        }
    }

    async fn shutdown_and_wait(self) {
        self.shutdown_token.cancel();

        if let Err(e) = self.handle.await {
            tracing::error!("Failed to join on account controller handle: {}", e);
        }
    }
}

async fn start_account_controller(data_dir: PathBuf) -> Result<AccountControllerHandle, VpnError> {
    // let data_path = config.credential_data_path.clone().unwrap();
    let storage = Arc::new(tokio::sync::Mutex::new(
        crate::storage::VpnClientOnDiskStorage::new(data_dir.clone()),
    ));
    // TODO: pass in as argument
    let user_agent = crate::util::construct_user_agent();
    let shutdown_token = CancellationToken::new();
    let account_controller = nym_vpn_account_controller::AccountController::new(
        Arc::clone(&storage),
        data_dir.clone(),
        user_agent,
        shutdown_token.child_token(),
    )
    .await;

    let shared_account_state = account_controller.shared_state();
    let account_command_tx = account_controller.command_tx();
    let account_controller_handle = tokio::spawn(account_controller.run());

    Ok(AccountControllerHandle {
        command_sender: account_command_tx,
        shared_state: shared_account_state,
        handle: account_controller_handle,
        shutdown_token,
    })
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

async fn start_state_machine(config: VPNConfig) -> Result<StateMachineHandle, VpnError> {
    let tunnel_type = if config.enable_two_hop {
        TunnelType::Wireguard
    } else {
        TunnelType::Mixnet
    };

    let entry_point = nym_gateway_directory::EntryPoint::from(config.entry_gateway);
    let exit_point = nym_gateway_directory::ExitPoint::from(config.exit_router);

    let gateway_config = GatewayDirectoryConfig {
        api_url: config.api_url,
        nym_vpn_api_url: config.vpn_api_url,
        ..Default::default()
    };

    // TODO: remove unwrap
    let data_dir = config.credential_data_path.clone().unwrap();

    let nym_config = NymConfig {
        // data_path: config.credential_data_path,
        data_path: Some(data_dir.clone()),
        gateway_config,
    };

    let tunnel_settings = TunnelSettings {
        tunnel_type,
        enable_credentials_mode: false,
        mixnet_tunnel_options: MixnetTunnelOptions::default(),
        gateway_performance_options: GatewayPerformanceOptions::default(),
        mixnet_client_config: None,
        entry_point: Box::new(entry_point),
        exit_point: Box::new(exit_point),
        dns: DnsOptions::default(),
    };

    let shutdown_token = CancellationToken::new();

    // let data_path = config.credential_data_path.clone().unwrap();
    //let storage = Arc::new(tokio::sync::Mutex::new(
    //    crate::storage::VpnClientOnDiskStorage::new(data_dir.clone()),
    //));
    //// TODO: pass in as argument
    //let user_agent = crate::util::construct_user_agent();
    //let account_controller = nym_vpn_account_controller::AccountController::new(
    //    Arc::clone(&storage),
    //    data_dir.clone(),
    //    user_agent,
    //    shutdown_token.child_token(),
    //)
    //.await;
    //let shared_account_state = account_controller.shared_state();
    //let account_command_tx = account_controller.command_tx();
    //let account_controller_handle = tokio::spawn(account_controller.run());

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

        // account_command_sender: account_command_tx,
        // account_shared_state: shared_account_state,
        // account_controller_handle,
        shutdown_token,
    })
}

impl From<&TunnelState> for TunStatus {
    fn from(value: &TunnelState) -> Self {
        // TODO: this cannot be accurate so we must switch frontends to use TunnelState instead! But for now that will do.
        match value {
            TunnelState::Connecting => Self::EstablishingConnection,
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
