// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod account;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod dns_handler;
mod ipv6_availability;
#[cfg(target_os = "macos")]
mod resolver;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod route_handler;
mod states;
#[cfg(any(target_os = "linux", target_os = "macos"))]
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
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};

use nym_config::defaults::{WG_METADATA_PORT, WG_TUN_DEVICE_IP_ADDRESS_V4};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_dns::ResolvedDnsConfig;
use nym_offline_monitor::ConnectivityHandle;
use nym_statistics::{StatisticsSender, events::StatisticsEvent};
use nym_vpn_account_controller::{AccountCommandSender, AccountStateReceiver};
use nym_vpn_network_config::Network;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_dns::DnsConfig;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_firewall::{Firewall, FirewallArguments, InitialFirewallState};
use nym_gateway_directory::{
    Config as GatewayDirectoryConfig, EntryPoint, ExitPoint, GatewayCacheHandle,
};
use nym_sdk::UserAgent;
use nym_vpn_lib_types::{
    AccountControllerErrorStateReason, ActionAfterDisconnect, ConnectionData, ErrorStateReason,
    EstablishConnectionData, EstablishConnectionState, MixnetEvent, TunnelEvent, TunnelState,
    TunnelType,
};
use nym_wg_gateway_client::Error as WgGatewayClientError;

use tunnel::SelectedGateways;
#[cfg(windows)]
use wintun::SetupWintunAdapterError;

#[cfg(target_os = "android")]
use crate::tunnel_provider::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::OSTunProvider;
use crate::{
    GatewayDirectoryError, MixnetClientConfig, MixnetError, VpnTopologyProvider,
    bandwidth_controller::Error as BandwidthControllerError,
};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use dns_handler::DnsHandlerHandle;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub use route_handler::RouteHandler;
#[cfg(target_os = "linux")]
pub use route_handler::RoutingParameters;
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TunnelConstants {
    /// Private (in-tunnel) entry gateway address
    pub private_entry_gateway_address: IpAddr,

    /// In-tunnel endpoint used for bandwidth queries
    pub in_tunnel_bandwidth_metadata_endpoint: SocketAddr,

    #[cfg(target_os = "linux")]
    /// Firewall mark used for bypassing the tunnel
    pub fwmark: u32,
}

impl Default for TunnelConstants {
    fn default() -> Self {
        Self {
            private_entry_gateway_address: IpAddr::from(WG_TUN_DEVICE_IP_ADDRESS_V4),
            in_tunnel_bandwidth_metadata_endpoint: SocketAddr::new(
                IpAddr::from(WG_TUN_DEVICE_IP_ADDRESS_V4),
                WG_METADATA_PORT,
            ),
            #[cfg(target_os = "linux")]
            fwmark: crate::TUNNEL_FWMARK,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TunnelSettings {
    /// Whether to enable support for IPv6.
    pub enable_ipv6: bool,

    /// Type of tunnel.
    pub tunnel_type: TunnelType,

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

impl TunnelSettings {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    /// Returns resolved DNS config resolved against default DNS IPs.
    pub fn resolved_dns_config(&self) -> ResolvedDnsConfig {
        self.dns.to_dns_config().resolve(
            &self.default_dns_ips(),
            #[cfg(target_os = "macos")]
            53,
        )
    }

    /// Returns DNS IPs filtering out IPv6 addresses when IPv6 is disabled.
    pub fn default_dns_ips(&self) -> Vec<IpAddr> {
        crate::DEFAULT_DNS_SERVERS
            .iter()
            .filter(|ip| ip.is_ipv4() || (ip.is_ipv6() && self.enable_ipv6))
            .copied()
            .collect()
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct GatewayPerformanceOptions {
    pub mixnet_min_performance: Option<u8>,
    pub vpn_min_performance: Option<u8>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct MixnetTunnelOptions {
    /// Overrides tunnel interface MTU.
    pub mtu: Option<u16>,

    /// Enable the credentials mode between the client and the gateways.
    pub enable_credentials_mode: bool,
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
            enable_ipv6: true,
            tunnel_type: TunnelType::Wireguard,
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
            PrivateTunnelState::Connecting {
                retry_attempt,
                state,
                tunnel_type,
                connection_data,
            } => Self::Connecting {
                retry_attempt,
                state,
                tunnel_type,
                connection_data,
            },
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
        /// Connection attempt.
        retry_attempt: u32,
        state: EstablishConnectionState,
        tunnel_type: TunnelType,
        connection_data: Option<EstablishConnectionData>,
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
            PrivateActionAfterDisconnect::Reconnect => Self::Reconnect,
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

    /// Reconnect after disconnect
    Reconnect,

    /// Enter offline state after disconnect
    Offline {
        /// Whether to reconnect the tunnel once back online.
        reconnect: bool,

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

impl TunnelInterface {
    /// Returns exit tunnel metadata
    pub fn exit_tunnel_metadata(&self) -> &TunnelMetadata {
        match self {
            Self::One(metadata) => metadata,
            Self::Two { exit, .. } => exit,
        }
    }
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
    connectivity_handle: nym_offline_monitor::ConnectivityHandle,
    /// Filtering resolver handle
    #[cfg(target_os = "macos")]
    filtering_resolver: resolver::ResolverHandle,
    nym_config: NymConfig,
    tunnel_settings: TunnelSettings,
    tunnel_constants: TunnelConstants,
    status_listener_handle: Option<JoinHandle<()>>,
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,
    account_command_tx: AccountCommandSender,
    account_controller_state: AccountStateReceiver,
    statistics_event_sender: StatisticsSender,
    gateway_cache_handle: GatewayCacheHandle,
    topology_provider: VpnTopologyProvider,
}

#[derive(Debug, Clone)]
pub struct NymConfig {
    pub config_path: Option<PathBuf>,
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
    #[cfg(target_os = "macos")]
    filtering_resolver_handle: JoinHandle<()>,
    shutdown_token: CancellationToken,
}

impl TunnelStateMachine {
    #[allow(clippy::too_many_arguments)]
    pub async fn spawn(
        command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
        event_sender: mpsc::UnboundedSender<TunnelEvent>,
        nym_config: NymConfig,
        tunnel_settings: TunnelSettings,
        tunnel_constants: TunnelConstants,
        account_command_tx: AccountCommandSender,
        account_controller_state: AccountStateReceiver,
        statistics_event_sender: StatisticsSender,
        gateway_cache_handle: GatewayCacheHandle,
        topology_provider: VpnTopologyProvider,
        connectivity_handle: ConnectivityHandle,
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))] route_handler: RouteHandler,
        #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<()>> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let dns_handler_shutdown_token = CancellationToken::new();

        #[cfg(target_os = "macos")]
        let (filtering_resolver, filtering_resolver_handle) =
            resolver::LocalResolver::spawn(true, dns_handler_shutdown_token.child_token())
                .await
                .map_err(Error::StartLocalDnsResolver)?;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let (dns_handler, dns_handler_task) = DnsHandlerHandle::spawn(
            #[cfg(target_os = "linux")]
            &route_handler,
            dns_handler_shutdown_token.child_token(),
        )
        .map_err(Error::CreateDnsHandler)?;

        let (mixnet_event_sender, mixnet_event_receiver) = mpsc::unbounded_channel();

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let firewall = Firewall::from_args(FirewallArguments {
            allow_lan: true,
            initial_state: InitialFirewallState::None,
            #[cfg(target_os = "linux")]
            fwmark: tunnel_constants.fwmark,
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
            connectivity_handle,
            #[cfg(target_os = "macos")]
            filtering_resolver,
            nym_config,
            tunnel_settings,
            tunnel_constants,
            status_listener_handle: None,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            tun_provider,
            account_command_tx,
            account_controller_state,
            statistics_event_sender,
            gateway_cache_handle,
            topology_provider,
        };

        let (current_state_handler, _) = if shared_state
            .connectivity_handle
            .connectivity()
            .await
            .is_offline()
        {
            OfflineState::enter(false, None, &mut shared_state).await
        } else {
            DisconnectedState::enter(None, &mut shared_state).await
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
            #[cfg(target_os = "macos")]
            filtering_resolver_handle,
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
                    if let Some(event) = StatisticsEvent::new_from_state(state.clone()) {
                        self.shared_state.statistics_event_sender.report(event)
                    }
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

        #[cfg(target_os = "macos")]
        {
            if let Err(e) = self.filtering_resolver_handle.await {
                tracing::error!("Failed to join on filtering resolver task: {}", e)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to create a route handler")]
    CreateRouteHandler(#[source] route_handler::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to create a dns handler")]
    CreateDnsHandler(#[source] dns_handler::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to create firewall")]
    CreateFirewall(#[source] nym_firewall::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to set firewall policy")]
    SetFirewallPolicy(#[source] nym_firewall::Error),

    #[error("failed to resolve API hostnames")]
    ResolveApiHostnames(#[source] Box<nym_gateway_directory::Error>),

    #[cfg(target_os = "macos")]
    #[error("failed to start local dns resolver")]
    StartLocalDnsResolver(#[source] resolver::Error),

    #[error("failed to create tunnel device")]
    CreateTunDevice(#[source] tun::Error),

    #[cfg(windows)]
    #[error("failed to setup wintun adapter")]
    SetupWintunAdapter(#[from] SetupWintunAdapterError),

    #[cfg(target_os = "ios")]
    #[error("failed to locate tun device")]
    LocateTunDevice(#[source] std::io::Error),

    #[cfg(any(target_os = "ios", target_os = "android"))]
    #[error("failed to configure tunnel provider: {}", _0)]
    ConfigureTunnelProvider(String),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to obtain route handle")]
    GetRouteHandle(#[source] route_handler::Error),

    #[error("failed to get tunnel device name")]
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    GetTunDeviceName(#[source] tun::Error),

    #[error("failed to get the interface IP sender")]
    GetInterfaceIpSender,

    #[error("failed to get tunnel device name")]
    #[cfg(any(target_os = "ios", target_os = "android"))]
    GetTunDeviceName(#[source] tun_name::GetTunNameError),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to set tunnel device ipv6 address")]
    SetTunDeviceIpv6Addr(#[source] std::io::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to add routes")]
    AddRoutes(#[source] route_handler::Error),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to set dns")]
    SetDns(#[source] dns_handler::Error),

    #[error("tunnel error")]
    Tunnel(#[from] Box<tunnel::Error>),

    #[error(transparent)]
    Account(#[from] account::Error),

    #[error("ipv6 is disabled in the system")]
    Ipv6Unavailable,
}

impl Error {
    fn error_state_reason(self) -> Option<ErrorStateReason> {
        Some(match self {
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::CreateRouteHandler(_) | Self::CreateDnsHandler(_) | Self::CreateFirewall(_) => {
                None?
            }
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::AddRoutes(_) => ErrorStateReason::SetRouting,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::SetDns(_) => ErrorStateReason::SetDns,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::SetFirewallPolicy(_) => ErrorStateReason::SetFirewallPolicy,
            Self::CreateTunDevice(_) => ErrorStateReason::TunDevice,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::SetTunDeviceIpv6Addr(_) => ErrorStateReason::TunDevice,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::GetTunDeviceName(_) => ErrorStateReason::TunDevice,
            Self::GetInterfaceIpSender => ErrorStateReason::Internal(self.to_string()),
            #[cfg(any(target_os = "ios", target_os = "android"))]
            Self::GetTunDeviceName(_) => ErrorStateReason::TunDevice,
            Self::ResolveApiHostnames(_) => None?,
            #[cfg(target_os = "macos")]
            Self::StartLocalDnsResolver(_) => None?,
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
            Self::Ipv6Unavailable => ErrorStateReason::Ipv6Unavailable,
        })
    }
}

impl tunnel::Error {
    fn error_state_reason(self) -> Option<ErrorStateReason> {
        match self {
            Self::SelectGateways(e) => match *e {
                GatewayDirectoryError::SameEntryAndExitGateway { .. } => {
                    Some(ErrorStateReason::SameEntryAndExitGateway)
                }
                GatewayDirectoryError::SelectEntryGateway(
                    nym_gateway_directory::Error::NoMatchingEntryGatewayForLocation { .. },
                ) => Some(ErrorStateReason::InvalidEntryGatewayCountry),
                GatewayDirectoryError::SelectExitGateway(
                    nym_gateway_directory::Error::NoMatchingExitGatewayForLocation { .. },
                ) => Some(ErrorStateReason::InvalidExitGatewayCountry),
                _ => None,
            },
            Self::BandwidthController(BandwidthControllerError::RegisterWireguard {
                source,
                ..
            }) => match *source {
                WgGatewayClientError::NoRetry { .. } => {
                    Some(ErrorStateReason::BadBandwidthIncrease)
                }
                _ => None,
            },
            Self::BandwidthController(BandwidthControllerError::RequestCredential {
                source,
                ..
            }) => match *source {
                WgGatewayClientError::NoRetry { .. } => {
                    Some(ErrorStateReason::BadBandwidthIncrease)
                }
                _ => None,
            },
            Self::DupFd(_) => Some(ErrorStateReason::Internal(
                "Failed to dup tunnel fd".to_owned(),
            )),
            Self::MixnetClient(MixnetError::CreateMixnetClientWithDefaultStorage(_)) => Some(
                ErrorStateReason::Internal("Failed to create mixnet storage".to_owned()),
            ),
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
            | Self::Cancelled
            | Self::MixnetClientDisposed => None,
            #[cfg(target_os = "ios")]
            Self::ResolveDns64(_) => None,
            #[cfg(windows)]
            Self::AddDefaultRouteListener(_) => None,
        }
    }
}

impl account::Error {
    fn error_state_reason(self) -> Option<ErrorStateReason> {
        use nym_vpn_lib_types::AccountControllerError as AcError;
        match self {
            Self::Command(e) => Some(ErrorStateReason::Internal(e.to_string())),
            Self::Cancelled => None,
            Self::ControllerState(e) => match e {
                AcError::Offline => None,
                AcError::NoAccountStored => Some(ErrorStateReason::DeviceLoggedOut),
                AcError::Internal(e) => Some(ErrorStateReason::Internal(e.to_string())),
                AcError::ErrorState(
                    AccountControllerErrorStateReason::AccountStatusNotActive { .. },
                ) => Some(ErrorStateReason::InactiveAccount),
                AcError::ErrorState(AccountControllerErrorStateReason::BandwidthExceeded {
                    ..
                }) => Some(ErrorStateReason::BandwidthExceeded),
                AcError::ErrorState(AccountControllerErrorStateReason::InactiveSubscription) => {
                    Some(ErrorStateReason::InactiveSubscription)
                }
                AcError::ErrorState(AccountControllerErrorStateReason::MaxDeviceReached) => {
                    Some(ErrorStateReason::MaxDevicesReached)
                }
                AcError::ErrorState(AccountControllerErrorStateReason::DeviceTimeDesynced) => {
                    Some(ErrorStateReason::DeviceTimeOutOfSync)
                }
                AcError::ErrorState(AccountControllerErrorStateReason::Internal {
                    context,
                    details,
                }) => Some(ErrorStateReason::Internal(format!(
                    "Internal account controller error: {context} {details}"
                ))),
                AcError::ErrorState(AccountControllerErrorStateReason::Storage { context }) => {
                    Some(ErrorStateReason::Internal(format!(
                        "Failed to initialize account storage: {context}",
                    )))
                }
                AcError::ErrorState(AccountControllerErrorStateReason::ApiFailure {
                    context,
                    details,
                }) => Some(ErrorStateReason::Internal(format!(
                    "Account API failure: {context} {details}"
                ))),
            },
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
