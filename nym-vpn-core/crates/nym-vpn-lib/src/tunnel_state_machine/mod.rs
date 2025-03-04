// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod account;
#[cfg(target_os = "android")]
mod android_connectivity_adapter;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod dns_handler;
#[cfg(target_os = "macos")]
mod resolver;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod route_handler;
mod states;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod tun_ipv6;
#[cfg(any(target_os = "ios", target_os = "android"))]
mod tun_name;
pub mod tunnel;
mod tunnel_monitor;
#[cfg(windows)]
mod wintun;

#[cfg(any(target_os = "ios", target_os = "android"))]
use std::sync::Arc;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};

use nym_vpn_account_controller::AccountControllerCommander;
use nym_vpn_network_config::Network;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_dns::DnsConfig;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_firewall::{Firewall, FirewallArguments, InitialFirewallState};
use nym_gateway_directory::{Config as GatewayDirectoryConfig, EntryPoint, ExitPoint, Recipient};
use nym_ip_packet_requests::IpPair;
use nym_sdk::UserAgent;
use nym_vpn_lib_types::{
    ActionAfterDisconnect, ConnectionData, ErrorStateReason, MixnetEvent, TunnelEvent, TunnelState,
    TunnelType,
};
use nym_wg_gateway_client::Error as WgGatewayClientError;

use tunnel::SelectedGateways;
#[cfg(windows)]
use wintun::SetupWintunAdapterError;

#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::ios::OSTunProvider;
use crate::{
    bandwidth_controller::Error as BandwidthControllerError, GatewayDirectoryError,
    MixnetClientConfig,
};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use dns_handler::DnsHandlerHandle;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use route_handler::RouteHandler;
use states::{DisconnectedState, OfflineState};

#[async_trait::async_trait]
trait TunnelStateHandler: Send {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState;
}

// todo: fix large enum; 248 byte enum is by no means a problem but clippy thinks we develop a firmware for Mars rovers.
#[allow(clippy::large_enum_variant)]
enum NextTunnelState {
    NewState((Box<dyn TunnelStateHandler>, PrivateTunnelState)),
    SameState(Box<dyn TunnelStateHandler>),
    Finished,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TunnelSettings {
    /// Type of tunnel.
    pub tunnel_type: TunnelType,

    /// Enable the credentials mode between the client and the gateways.
    pub enable_credentials_mode: bool,

    /// The (optional) recipient to send statistics to.
    pub statistics_recipient: Option<Box<Recipient>>,

    /// Mixnet tunnel options.
    pub mixnet_tunnel_options: MixnetTunnelOptions,

    /// WireGuard tunnel options.
    pub wireguard_tunnel_options: WireguardTunnelOptions,

    /// Overrides gateway config.
    pub gateway_performance_options: GatewayPerformanceOptions,

    /// Overrides mixnet client config when provided.
    /// Leave `None` to use sane defaults.
    pub mixnet_client_config: Option<MixnetClientConfig>,

    /// Entry node.
    pub entry_point: Box<EntryPoint>,

    /// Exit node.
    pub exit_point: Box<ExitPoint>,

    /// DNS configuration.
    pub dns: DnsOptions,

    /// The user agent used for HTTP requests.
    pub user_agent: Option<UserAgent>,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct GatewayPerformanceOptions {
    pub mixnet_min_performance: Option<u8>,
    pub vpn_min_performance: Option<u8>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct MixnetTunnelOptions {
    /// Overrides tunnel interface addresses.
    pub interface_addrs: Option<IpPair>,

    /// Overrides tunnel interface MTU.
    pub mtu: Option<u16>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WireguardMultihopMode {
    /// Multihop using two tun devices to nest tunnels.
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    TunTun,

    /// Netstack based multihop.
    Netstack,
}

impl Default for WireguardMultihopMode {
    fn default() -> Self {
        #[cfg(any(target_os = "ios", target_os = "android"))]
        {
            Self::Netstack
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            Self::TunTun
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct WireguardTunnelOptions {
    pub multihop_mode: WireguardMultihopMode,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum DnsOptions {
    #[default]
    Default,
    Custom(Vec<IpAddr>),
}

impl DnsOptions {
    /// Convert dns options into [DnsConfig].
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn to_dns_config(&self) -> DnsConfig {
        match self {
            Self::Default => DnsConfig::default(),
            Self::Custom(addrs) => {
                if addrs.is_empty() {
                    DnsConfig::default()
                } else {
                    let (non_tunnel_config, tunnel_config): (Vec<_>, Vec<_>) = addrs
                        .iter()
                        // Private IP ranges should not be tunneled
                        .partition(|&addr| nym_firewall::is_local_address(addr));
                    DnsConfig::from_addresses(&tunnel_config, &non_tunnel_config)
                }
            }
        }
    }

    #[cfg(any(target_os = "ios", target_os = "android"))]
    pub fn ip_addresses<'a>(&'a self, default_addresses: &'a [IpAddr]) -> &'a [IpAddr] {
        match self {
            Self::Default => default_addresses,
            Self::Custom(addrs) => addrs.as_slice(),
        }
    }
}

impl Default for TunnelSettings {
    fn default() -> Self {
        Self {
            tunnel_type: TunnelType::Wireguard,
            enable_credentials_mode: false,
            statistics_recipient: None,
            mixnet_tunnel_options: MixnetTunnelOptions::default(),
            mixnet_client_config: None,
            wireguard_tunnel_options: WireguardTunnelOptions::default(),
            gateway_performance_options: GatewayPerformanceOptions::default(),
            entry_point: Box::new(EntryPoint::Random),
            exit_point: Box::new(ExitPoint::Random),
            dns: DnsOptions::default(),
            user_agent: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TunnelCommand {
    /// Connect the tunnel.
    Connect,

    /// Disconnect the tunnel.
    Disconnect,

    /// Set new tunnel settings.
    SetTunnelSettings(TunnelSettings),
}

impl From<PrivateTunnelState> for TunnelState {
    fn from(value: PrivateTunnelState) -> Self {
        match value {
            PrivateTunnelState::Disconnected => Self::Disconnected,
            PrivateTunnelState::Connected { connection_data } => {
                Self::Connected { connection_data }
            }
            PrivateTunnelState::Connecting { connection_data } => {
                Self::Connecting { connection_data }
            }
            PrivateTunnelState::Disconnecting { after_disconnect } => Self::Disconnecting {
                after_disconnect: ActionAfterDisconnect::from(after_disconnect),
            },
            PrivateTunnelState::Error(reason) => Self::Error(reason),
            PrivateTunnelState::Offline { reconnect } => Self::Offline { reconnect },
        }
    }
}

/// Private enum describing the tunnel state
#[derive(Debug, Clone)]
enum PrivateTunnelState {
    Disconnected,
    Connecting {
        connection_data: Option<ConnectionData>,
    },
    Connected {
        connection_data: ConnectionData,
    },
    Disconnecting {
        after_disconnect: PrivateActionAfterDisconnect,
    },
    Error(ErrorStateReason),
    Offline {
        /// Whether to reconnect after gaining the network connectivity.
        reconnect: bool,
    },
}

impl From<PrivateActionAfterDisconnect> for ActionAfterDisconnect {
    fn from(value: PrivateActionAfterDisconnect) -> Self {
        match value {
            PrivateActionAfterDisconnect::Nothing => Self::Nothing,
            PrivateActionAfterDisconnect::Reconnect { .. } => Self::Reconnect,
            PrivateActionAfterDisconnect::Offline { .. } => Self::Offline,
            PrivateActionAfterDisconnect::Error(_) => Self::Error,
        }
    }
}

/// Private enum describing action to perform after disconnect
#[derive(Debug, Clone)]
enum PrivateActionAfterDisconnect {
    /// Do nothing after disconnect
    Nothing,

    /// Reconnect after disconnect, providing the retry attempt counter
    Reconnect { retry_attempt: u32 },

    /// Enter offline state after disconnect
    Offline {
        /// Whether to reconnect the tunnel once back online.
        reconnect: bool,

        /// The last recorded retry attempt passed to connecting state upon reconnect.
        retry_attempt: u32,

        /// The last known gateways passed to connecting state upon reconnect.
        gateways: Option<SelectedGateways>,
    },

    /// Enter error state
    Error(ErrorStateReason),
}

/// Describes tunnel interfaces used to maintain the tunnel.
#[derive(Debug, Clone)]
pub enum TunnelInterface {
    One(TunnelMetadata),
    Two {
        entry: TunnelMetadata,
        exit: TunnelMetadata,
    },
}

/// Describes tunnel interface configuration.
#[derive(Debug, Clone)]
#[cfg_attr(any(target_os = "ios", target_os = "android"), allow(unused))]
pub struct TunnelMetadata {
    interface: String,
    ips: Vec<IpAddr>,
    ipv4_gateway: Option<Ipv4Addr>,
    ipv6_gateway: Option<Ipv6Addr>,
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
impl From<TunnelMetadata> for nym_firewall::TunnelMetadata {
    fn from(value: TunnelMetadata) -> Self {
        Self {
            interface: value.interface,
            ips: value.ips,
            ipv4_gateway: value.ipv4_gateway,
            ipv6_gateway: value.ipv6_gateway,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
impl From<TunnelInterface> for nym_firewall::TunnelInterface {
    fn from(value: TunnelInterface) -> Self {
        match value {
            TunnelInterface::One(metadata) => {
                nym_firewall::TunnelInterface::One(nym_firewall::TunnelMetadata::from(metadata))
            }
            TunnelInterface::Two { entry, exit } => nym_firewall::TunnelInterface::Two {
                entry: nym_firewall::TunnelMetadata::from(entry),
                exit: nym_firewall::TunnelMetadata::from(exit),
            },
        }
    }
}

pub struct SharedState {
    mixnet_event_sender: mpsc::UnboundedSender<MixnetEvent>,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    route_handler: RouteHandler,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    firewall: Firewall,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    dns_handler: DnsHandlerHandle,
    offline_monitor: nym_offline_monitor::MonitorHandle,
    /// Filtering resolver handle
    #[cfg(target_os = "macos")]
    filtering_resolver: resolver::ResolverHandle,
    nym_config: NymConfig,
    tunnel_settings: TunnelSettings,
    status_listener_handle: Option<JoinHandle<()>>,
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,
    account_command_tx: AccountControllerCommander,
}

#[derive(Debug, Clone)]
pub struct NymConfig {
    pub data_path: Option<PathBuf>,
    pub gateway_config: GatewayDirectoryConfig,
    pub network_env: Network,
}

pub struct TunnelStateMachine {
    current_state_handler: Box<dyn TunnelStateHandler>,
    shared_state: SharedState,
    command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
    event_sender: mpsc::UnboundedSender<TunnelEvent>,
    mixnet_event_receiver: mpsc::UnboundedReceiver<MixnetEvent>,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    dns_handler_task: JoinHandle<()>,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    dns_handler_shutdown_token: CancellationToken,
    shutdown_token: CancellationToken,
}

impl TunnelStateMachine {
    pub async fn spawn(
        command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
        event_sender: mpsc::UnboundedSender<TunnelEvent>,
        nym_config: NymConfig,
        tunnel_settings: TunnelSettings,
        account_command_tx: AccountControllerCommander,
        #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<()>> {
        #[cfg(target_os = "macos")]
        let filtering_resolver = resolver::start_resolver()
            .await
            .map_err(Error::StartLocalDnsResolver)?;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let route_handler = RouteHandler::new()
            .await
            .map_err(Error::CreateRouteHandler)?;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let dns_handler_shutdown_token = CancellationToken::new();
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let (dns_handler, dns_handler_task) = DnsHandlerHandle::spawn(
            #[cfg(target_os = "linux")]
            &route_handler,
            dns_handler_shutdown_token.child_token(),
        )
        .map_err(Error::CreateDnsHandler)?;

        let offline_monitor = nym_offline_monitor::spawn_monitor(
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_handler.inner_handle(),
            #[cfg(target_os = "android")]
            android_connectivity_adapter::AndroidConnectivityAdapter::new(tun_provider.clone()),
            #[cfg(target_os = "linux")]
            Some(route_handler::TUNNEL_FWMARK),
        )
        .await;

        let (mixnet_event_sender, mixnet_event_receiver) = mpsc::unbounded_channel();

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let firewall = Firewall::from_args(FirewallArguments {
            allow_lan: true,
            initial_state: InitialFirewallState::None,
            #[cfg(target_os = "linux")]
            fwmark: route_handler::TUNNEL_FWMARK,
        })
        .map_err(Error::CreateFirewall)?;

        let mut shared_state = SharedState {
            mixnet_event_sender,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            route_handler,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            firewall,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            dns_handler,
            offline_monitor,
            #[cfg(target_os = "macos")]
            filtering_resolver,
            nym_config,
            tunnel_settings,
            status_listener_handle: None,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            tun_provider,
            account_command_tx,
        };

        let (current_state_handler, _) = if shared_state
            .offline_monitor
            .connectivity()
            .await
            .is_offline()
        {
            OfflineState::enter(false, 0, None, &mut shared_state).await
        } else {
            DisconnectedState::enter(&mut shared_state).await
        };

        let tunnel_state_machine = Self {
            current_state_handler,
            shared_state,
            command_receiver,
            event_sender,
            mixnet_event_receiver,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            dns_handler_task,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            dns_handler_shutdown_token,
            shutdown_token,
        };

        Ok(tokio::spawn(tunnel_state_machine.run()))
    }

    async fn run(mut self) {
        let mut mixnet_event_receiver = self.mixnet_event_receiver;
        let cloned_event_sender = self.event_sender.clone();
        tokio::spawn(async move {
            while let Some(event) = mixnet_event_receiver.recv().await {
                if let Err(e) = cloned_event_sender.send(TunnelEvent::MixnetState(event)) {
                    tracing::error!("Failed to send tunnel event: {}", e);
                }
            }
        });

        loop {
            let next_state = self
                .current_state_handler
                .handle_event(
                    &self.shutdown_token,
                    &mut self.command_receiver,
                    &mut self.shared_state,
                )
                .await;

            match next_state {
                NextTunnelState::NewState((new_state_handler, new_state)) => {
                    self.current_state_handler = new_state_handler;

                    let state = TunnelState::from(new_state);
                    tracing::info!("New tunnel state: {}", state);
                    let _ = self.event_sender.send(TunnelEvent::NewState(state));
                }
                NextTunnelState::SameState(same_state) => {
                    self.current_state_handler = same_state;
                }
                NextTunnelState::Finished => break,
            }
        }

        tracing::debug!("Tunnel state machine is exiting...");

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            self.dns_handler_shutdown_token.cancel();
            if let Err(e) = self.dns_handler_task.await {
                tracing::error!("Failed to join on dns handler task: {}", e)
            }

            self.shared_state.route_handler.stop().await;
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to create a route handler: {}", _0)]
    CreateRouteHandler(#[source] route_handler::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to create a dns handler: {}", _0)]
    CreateDnsHandler(#[source] dns_handler::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to create firewall: {}", _0)]
    CreateFirewall(#[source] nym_firewall::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to apply firewall policy: {}", _0)]
    ApplyFirewallPolicy(#[source] nym_firewall::Error),

    #[error("failed to resolve gateway addresses: {}", _0)]
    ResolveGatewayAddrs(#[source] nym_gateway_directory::Error),

    #[cfg(target_os = "macos")]
    #[error("failed to start local dns resolver: {}", _0)]
    StartLocalDnsResolver(#[source] resolver::Error),

    #[error("failed to create tunnel device: {}", _0)]
    CreateTunDevice(#[source] tun::Error),

    #[cfg(windows)]
    #[error("failed to setup wintun adapter: {}", _0)]
    SetupWintunAdapter(#[from] SetupWintunAdapterError),

    #[cfg(target_os = "ios")]
    #[error("failed to locate tun device")]
    LocateTunDevice(#[source] std::io::Error),

    #[cfg(any(target_os = "ios", target_os = "android"))]
    #[error("failed to configure tunnel provider: {}", _0)]
    ConfigureTunnelProvider(String),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to obtain route handle: {}", _0)]
    GetRouteHandle(#[source] route_handler::Error),

    #[error("failed to get tunnel device name")]
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    GetTunDeviceName(#[source] tun::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to set tunnel device ipv6 address")]
    SetTunDeviceIpv6Addr(#[source] std::io::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to add routes: {}", _0)]
    AddRoutes(#[source] route_handler::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to set dns: {}", _0)]
    SetDns(#[source] dns_handler::Error),

    #[error("tunnel error: {}", _0)]
    Tunnel(#[from] tunnel::Error),

    #[error(transparent)]
    Account(#[from] account::Error),
}

impl Error {
    fn error_state_reason(self) -> Option<ErrorStateReason> {
        Some(match self {
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::CreateRouteHandler(_) | Self::AddRoutes(_) => ErrorStateReason::Routing,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::CreateDnsHandler(_) | Self::SetDns(_) => ErrorStateReason::Dns,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::CreateFirewall(_) | Self::ApplyFirewallPolicy(_) => ErrorStateReason::Firewall,
            Self::CreateTunDevice(_) => ErrorStateReason::TunDevice,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::SetTunDeviceIpv6Addr(_) => ErrorStateReason::TunDevice,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::GetTunDeviceName(_) => ErrorStateReason::TunDevice,
            Self::ResolveGatewayAddrs(_) => ErrorStateReason::ResolveGatewayAddrs,
            #[cfg(target_os = "macos")]
            Self::StartLocalDnsResolver(_) => ErrorStateReason::StartLocalDnsResolver,
            #[cfg(windows)]
            Self::SetupWintunAdapter(_) => ErrorStateReason::TunDevice,
            Self::Tunnel(e) => e.error_state_reason()?,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            Self::ConfigureTunnelProvider(_) => ErrorStateReason::TunnelProvider,
            #[cfg(target_os = "ios")]
            Self::LocateTunDevice(_) => ErrorStateReason::TunDevice,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::GetRouteHandle(e) => ErrorStateReason::Internal(e.to_string()),
            Self::Account(err) => err.error_state_reason()?,
        })
    }
}

impl tunnel::Error {
    fn error_state_reason(self) -> Option<ErrorStateReason> {
        match self {
            Self::SelectGateways(e) => match e {
                GatewayDirectoryError::SameEntryAndExitGateway { .. } => {
                    Some(ErrorStateReason::SameEntryAndExitGateway)
                }
                GatewayDirectoryError::FailedToSelectEntryGateway {
                    source: nym_gateway_directory::Error::NoMatchingEntryGatewayForLocation { .. },
                } => Some(ErrorStateReason::InvalidEntryGatewayCountry),
                GatewayDirectoryError::FailedToSelectExitGateway {
                    source: nym_gateway_directory::Error::NoMatchingExitGatewayForLocation { .. },
                } => Some(ErrorStateReason::InvalidExitGatewayCountry),
                _ => None,
            },
            Self::BandwidthController(BandwidthControllerError::RegisterWireguard {
                source: WgGatewayClientError::NoRetry { .. },
                ..
            })
            | Self::BandwidthController(BandwidthControllerError::TopUpWireguard {
                source: WgGatewayClientError::NoRetry { .. },
                ..
            }) => Some(ErrorStateReason::BadBandwidthIncrease),
            Self::DupFd(_) => Some(ErrorStateReason::DuplicateTunFd),
            Self::AuthenticationNotPossible(_)
            | Self::AuthenticatorAddressNotFound
            | Self::ConnectToIpPacketRouter(_)
            | Self::LookupGatewayIp { .. }
            | Self::MixnetClient(_)
            | Self::SetupStoragePaths(_)
            | Self::StartMixnetClientTimeout
            | Self::CreateGatewayClient(_)
            | Self::BandwidthController(_)
            | Self::Wireguard(_)
            | Self::Cancelled => None,
            #[cfg(target_os = "ios")]
            Self::ResolveDns64(_) => None,
            #[cfg(windows)]
            Self::AddDefaultRouteListener(_) => None,
        }
    }
}

impl account::Error {
    fn error_state_reason(self) -> Option<ErrorStateReason> {
        match self {
            Self::SyncAccount(e) => Some(e.into()),
            Self::SyncDevice(e) => Some(e.into()),
            Self::RegisterDevice(e) => Some(e.into()),
            Self::RequestZkNym(e) => Some(e.into()),
            Self::Cancelled => None,
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
