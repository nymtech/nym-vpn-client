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
use nym_ip_packet_requests::IpPair;
use tokio::{
    runtime::Runtime,
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use url::Url;

use nym_gateway_directory::Config as GatewayDirectoryConfig;
use nym_vpn_api_client::types::VpnApiAccount;
use nym_vpn_store::mnemonic::MnemonicStorage as _;

use self::error::VpnError;
#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::ios::OSTunProvider;
use crate::{
    gateway_directory::GatewayClient,
    tunnel_state_machine::{
        BandwidthEvent, ConnectionEvent, DnsOptions, GatewayPerformanceOptions, MixnetEvent,
        MixnetTunnelOptions, NymConfig, TunnelCommand, TunnelConnectionData, TunnelEvent,
        TunnelSettings, TunnelState, TunnelStateMachine, TunnelType,
    },
    uniffi_custom_impls::{
        BandwidthStatus, ConnectionStatus, EntryPoint, ExitPoint, ExitStatus,
        GatewayMinPerformance, GatewayType, Location, MixConnectionInfo, MixExitConnectionInfo,
        NymVpnStatus, TunStatus, UserAgent, WireguardConnectionInfo,
    },
};

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref STATE_MACHINE_HANDLE: Mutex<Option<StateMachineHandle>> = Mutex::new(None);
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

#[allow(dead_code)]
async fn get_account_summary(
    path: String,
    nym_vpn_api_url: Url,
    user_agent: UserAgent,
) -> Result<String, VpnError> {
    let storage = setup_account_storage(&path)?;
    let account = storage
        .load_mnemonic()
        .await
        .map(VpnApiAccount::from)
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

    let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent.into())?;

    api_client
        .get_account_summary(&account)
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })
        .and_then(|summary| {
            serde_json::to_string(&summary).map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })
        })
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
    fn on_tun_status_change(&self, status: TunStatus);
    fn on_bandwidth_status_change(&self, status: BandwidthStatus);
    fn on_connection_status_change(&self, status: ConnectionStatus);
    fn on_nym_vpn_status_change(&self, status: NymVpnStatus);
    fn on_exit_status_change(&self, status: ExitStatus);
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

    let nym_config = NymConfig {
        data_path: config.credential_data_path,
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

    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let (event_sender, mut event_receiver) = mpsc::unbounded_channel();

    let state_listener = config.tun_status_listener;
    let event_broadcaster_handler = tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            if let Some(ref state_listener) = state_listener {
                // todo: done this way for compatibility. New code should use on_event() instead.
                match event {
                    TunnelEvent::NewState(ref state) => {
                        if let Some(nym_vpn_status) = nym_vpn_status_from_tunnel_state(state) {
                            (*state_listener).on_nym_vpn_status_change(nym_vpn_status);
                        }

                        if let Some(exit_status) = exit_status_from_tunnel_state(state) {
                            (*state_listener).on_exit_status_change(exit_status);
                        }

                        (*state_listener).on_tun_status_change(TunStatus::from(state));
                    }
                    TunnelEvent::MixnetState(MixnetEvent::Bandwidth(sub_event)) => {
                        (*state_listener)
                            .on_bandwidth_status_change(BandwidthStatus::from(sub_event))
                    }
                    TunnelEvent::MixnetState(MixnetEvent::Connection(sub_event)) => {
                        (*state_listener)
                            .on_connection_status_change(ConnectionStatus::from(sub_event));
                    }
                }
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

fn exit_status_from_tunnel_state(value: &TunnelState) -> Option<ExitStatus> {
    match value {
        TunnelState::Disconnected => Some(ExitStatus::Stopped),
        TunnelState::Error(reason) => Some(ExitStatus::Failure {
            error: VpnError::InternalError {
                details: format!("{:?}", reason),
            },
        }),
        TunnelState::Disconnecting { .. }
        | TunnelState::Connecting
        | TunnelState::Connected { .. } => None,
    }
}

fn nym_vpn_status_from_tunnel_state(value: &TunnelState) -> Option<NymVpnStatus> {
    match value {
        TunnelState::Connected { connection_data } => Some(match &connection_data.tunnel {
            TunnelConnectionData::Mixnet(mixnet_data) => NymVpnStatus::MixConnectInfo {
                mix_connection_info: MixConnectionInfo {
                    nym_address: *mixnet_data.nym_address,
                    entry_gateway: *connection_data.entry_gateway,
                },
                mix_exit_connection_info: MixExitConnectionInfo {
                    exit_gateway: *connection_data.exit_gateway,
                    exit_ipr: *mixnet_data.exit_ipr,
                    ips: IpPair {
                        ipv4: mixnet_data.ipv4,
                        ipv6: mixnet_data.ipv6,
                    },
                },
            },
            TunnelConnectionData::Wireguard(data) => NymVpnStatus::WgConnectInfo {
                entry_connection_info: WireguardConnectionInfo {
                    gateway_id: *connection_data.entry_gateway,
                    public_key: data.entry.public_key.to_base64(),
                    private_ipv4: data.entry.private_ipv4,
                },
                exit_connection_info: WireguardConnectionInfo {
                    gateway_id: *connection_data.exit_gateway,
                    public_key: data.exit.public_key.to_base64(),
                    private_ipv4: data.exit.private_ipv4,
                },
            },
        }),
        TunnelState::Connecting
        | TunnelState::Disconnected
        | TunnelState::Disconnecting { .. }
        | TunnelState::Error(_) => None,
    }
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
