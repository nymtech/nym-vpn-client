// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
mod default_interface;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod dns_handler;
//mod firewall_handler;
#[cfg(target_os = "android")]
mod android_connectivity_adapter;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod route_handler;
mod states;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod tun_ipv6;
pub mod tunnel;
mod tunnel_monitor;
#[cfg(windows)]
mod wintun;

#[cfg(any(target_os = "ios", target_os = "android"))]
use std::sync::Arc;
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};

use si_scale::helpers::bibytes2;
use time::OffsetDateTime;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use nym_gateway_directory::{
    Config as GatewayDirectoryConfig, EntryPoint, ExitPoint, NodeIdentity, Recipient,
};
use nym_ip_packet_requests::IpPair;
use nym_sdk::UserAgent;
use nym_wg_gateway_client::{Error as WgGatewayClientError, GatewayData};
use nym_wg_go::PublicKey;

use tunnel::SelectedGateways;
#[cfg(windows)]
use wintun::SetupWintunAdapterError;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use dns_handler::DnsHandlerHandle;
//use firewall_handler::FirewallHandler;
#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::ios::OSTunProvider;
use crate::{
    bandwidth_controller::Error as BandwidthControllerError, GatewayDirectoryError,
    MixnetClientConfig,
};
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

enum NextTunnelState {
    NewState((Box<dyn TunnelStateHandler>, PrivateTunnelState)),
    SameState(Box<dyn TunnelStateHandler>),
    Finished,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, uniffi::Enum)]
pub enum TunnelType {
    Mixnet,
    Wireguard,
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
    fn ip_addresses(&self) -> &[IpAddr] {
        match self {
            Self::Default => &crate::DEFAULT_DNS_SERVERS,
            Self::Custom(addrs) => addrs,
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

#[derive(Clone, Eq, PartialEq, uniffi::Record)]
pub struct ConnectionData {
    /// Mixnet entry gateway
    pub entry_gateway: Box<NodeIdentity>,

    /// Mixnet exit gateway
    pub exit_gateway: Box<NodeIdentity>,

    /// When the tunnel was last established.
    /// Set once the tunnel is connected.
    pub connected_at: Option<OffsetDateTime>,

    /// Tunnel connection data.
    pub tunnel: TunnelConnectionData,
}

impl fmt::Debug for ConnectionData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionData")
            .field("entry_gateway", &self.entry_gateway.to_base58_string())
            .field("exit_gateway", &self.exit_gateway.to_base58_string())
            .field("connected_at", &self.connected_at)
            .field("tunnel", &self.tunnel)
            .finish()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Enum)]
pub enum TunnelConnectionData {
    Mixnet(MixnetConnectionData),
    Wireguard(WireguardConnectionData),
}

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Record)]
pub struct MixnetConnectionData {
    pub nym_address: Box<Recipient>,
    pub exit_ipr: Box<Recipient>,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
}

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Record)]
pub struct WireguardNode {
    pub endpoint: SocketAddr,
    pub public_key: Box<PublicKey>,
    pub private_ipv4: Ipv4Addr,
    pub private_ipv6: Ipv6Addr,
}

impl From<GatewayData> for WireguardNode {
    fn from(value: GatewayData) -> Self {
        Self {
            endpoint: value.endpoint,
            public_key: Box::new(value.public_key),
            private_ipv4: value.private_ipv4,
            private_ipv6: value.private_ipv6,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Record)]
pub struct WireguardConnectionData {
    pub entry: WireguardNode,
    pub exit: WireguardNode,
}

/// Public enum describing the tunnel state
#[derive(Debug, Clone, Eq, PartialEq, uniffi::Enum)]
pub enum TunnelState {
    /// Tunnel is disconnected and network connectivity is available.
    Disconnected,

    /// Tunnel connection is being established.
    Connecting {
        connection_data: Option<ConnectionData>,
    },

    /// Tunnel is connected.
    Connected { connection_data: ConnectionData },

    /// Tunnel is disconnecting.
    Disconnecting {
        after_disconnect: ActionAfterDisconnect,
    },

    /// Tunnel is disconnected due to failure.
    Error(ErrorStateReason),

    /// Tunnel is disconnected, network connectivity is unavailable.
    Offline {
        /// Whether tunnel will be reconnected upon gaining the network connectivity.
        reconnect: bool,
    },
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

/// Public enum describing action to perform after disconnect
#[derive(Debug, Clone, Copy, Eq, PartialEq, uniffi::Enum)]
pub enum ActionAfterDisconnect {
    /// Do nothing after disconnect
    Nothing,

    /// Reconnect after disconnect
    Reconnect,

    /// Enter offline after disconnect
    Offline,

    /// Enter error state
    Error,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, uniffi::Enum)]
pub enum ErrorStateReason {
    /// Issues related to firewall configuration.
    Firewall,

    /// Failure to configure routing.
    Routing,

    /// Failure to configure dns.
    Dns,

    /// Failure to configure tunnel device.
    TunDevice,

    /// Failure to configure packet tunnel provider.
    TunnelProvider,

    /// Same entry and exit gateway are unsupported.
    SameEntryAndExitGateway,

    /// Invalid country set for entry gateway
    InvalidEntryGatewayCountry,

    /// Invalid country set for exit gateway
    InvalidExitGatewayCountry,

    /// Gateway is not responding or responding badly to a bandwidth
    /// increase request, causing credential waste
    BadBandwidthIncrease,

    /// Failure to duplicate tunnel file descriptor.
    DuplicateTunFd,

    /// Program errors that must not happen.
    Internal,
}

#[derive(Debug, uniffi::Enum)]
pub enum TunnelEvent {
    NewState(TunnelState),
    MixnetState(MixnetEvent),
}

impl fmt::Display for TunnelEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NewState(new_state) => new_state.fmt(f),
            Self::MixnetState(event) => event.fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone, uniffi::Enum)]
pub enum MixnetEvent {
    Bandwidth(BandwidthEvent),
    Connection(ConnectionEvent),
    ConnectionStatistics(ConnectionStatisticsEvent),
}

#[derive(Debug, Copy, Clone, uniffi::Enum)]
pub enum BandwidthEvent {
    NoBandwidth,
    RemainingBandwidth(i64),
}

#[derive(Debug, Copy, Clone, uniffi::Enum)]
pub enum ConnectionEvent {
    EntryGatewayDown,
    ExitGatewayDownIpv4,
    ExitGatewayDownIpv6,
    ExitGatewayRoutingErrorIpv4,
    ExitGatewayRoutingErrorIpv6,
    ConnectedIpv4,
    ConnectedIpv6,
}

#[derive(Debug, Copy, Clone, uniffi::Record)]
pub struct ConnectionStatisticsEvent {
    pub rates: SphinxPacketRates,
}

#[derive(Debug, Copy, Clone, uniffi::Record)]
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

impl fmt::Display for ConnectionStatisticsEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rates)
    }
}

impl fmt::Display for SphinxPacketRates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

impl SphinxPacketRates {
    pub fn summary(&self) -> String {
        format!(
            "down: {}/s, up: {}/s (cover down: {}/s, cover up: {}/s)",
            bibytes2(self.real_packets_received_size),
            bibytes2(self.real_packets_sent_size),
            bibytes2(self.cover_packets_received_size),
            bibytes2(self.cover_packets_sent_size),
        )
    }

    pub fn real_received(&self) -> String {
        bibytes2(self.real_packets_received_size)
    }

    pub fn real_sent(&self) -> String {
        bibytes2(self.real_packets_sent_size)
    }

    pub fn cover_received(&self) -> String {
        bibytes2(self.cover_packets_received_size)
    }

    pub fn cover_sent(&self) -> String {
        bibytes2(self.cover_packets_sent_size)
    }
}

pub struct SharedState {
    mixnet_event_sender: mpsc::UnboundedSender<MixnetEvent>,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    route_handler: RouteHandler,
    //firewall_handler: FirewallHandler,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    dns_handler: DnsHandlerHandle,
    offline_monitor: nym_offline_monitor::MonitorHandle,
    nym_config: NymConfig,
    tunnel_settings: TunnelSettings,
    status_listener_handle: Option<JoinHandle<()>>,
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,
}

#[derive(Debug, Clone)]
pub struct NymConfig {
    pub data_path: Option<PathBuf>,
    pub gateway_config: GatewayDirectoryConfig,
}

pub struct TunnelStateMachine {
    current_state_handler: Box<dyn TunnelStateHandler>,
    shared_state: SharedState,
    command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
    event_sender: mpsc::UnboundedSender<TunnelEvent>,
    mixnet_event_receiver: mpsc::UnboundedReceiver<MixnetEvent>,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    dns_handler_task: JoinHandle<()>,
    shutdown_token: CancellationToken,
}

impl TunnelStateMachine {
    pub async fn spawn(
        command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
        event_sender: mpsc::UnboundedSender<TunnelEvent>,
        nym_config: NymConfig,
        tunnel_settings: TunnelSettings,
        #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<()>> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let route_handler = RouteHandler::new()
            .await
            .map_err(Error::CreateRouteHandler)?;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let (dns_handler, dns_handler_task) = DnsHandlerHandle::spawn(
            #[cfg(target_os = "linux")]
            &route_handler,
            shutdown_token.child_token(),
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

        let (current_state_handler, _) = if offline_monitor.connectivity().await.is_offline() {
            OfflineState::enter(false, 0, None)
        } else {
            DisconnectedState::enter()
        };

        //let firewall_handler = FirewallHandler::new().map_err(Error::CreateFirewallHandler)?;

        let (mixnet_event_sender, mixnet_event_receiver) = mpsc::unbounded_channel();

        let shared_state: SharedState = SharedState {
            mixnet_event_sender,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            route_handler,
            //firewall_handler,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            dns_handler,
            offline_monitor,
            nym_config,
            tunnel_settings,
            status_listener_handle: None,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            tun_provider,
        };

        let tunnel_state_machine = Self {
            current_state_handler,
            shared_state,
            command_receiver,
            event_sender,
            mixnet_event_receiver,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            dns_handler_task,
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
                    tracing::debug!("New tunnel state: {}", state);
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
        if let Err(e) = self.dns_handler_task.await {
            tracing::error!("Failed to join on dns handler task: {}", e)
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        self.shared_state.route_handler.stop().await;
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

    //#[error("failed to create firewall handler: {}", _0)]
    //CreateFirewallHandler(#[source] firewall_handler::Error),
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

    #[cfg(target_os = "linux")]
    #[error("failed to obtain default interface: {}", _0)]
    GetDefaultInterface(String),

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[error("failed to get tunnel device name")]
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
}

impl Error {
    fn error_state_reason(&self) -> Option<ErrorStateReason> {
        Some(match self {
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::CreateRouteHandler(_) | Self::AddRoutes(_) => ErrorStateReason::Routing,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::CreateDnsHandler(_) | Self::SetDns(_) => ErrorStateReason::Dns,
            //Self::CreateFirewallHandler(_) => ErrorStateReason::Firewall,
            Self::CreateTunDevice(_) => ErrorStateReason::TunDevice,

            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::GetTunDeviceName(_) | Self::SetTunDeviceIpv6Addr(_) => {
                ErrorStateReason::TunDevice
            }

            #[cfg(windows)]
            Self::SetupWintunAdapter(_) => ErrorStateReason::TunDevice,

            Self::Tunnel(e) => e.error_state_reason()?,

            #[cfg(any(target_os = "ios", target_os = "android"))]
            Self::ConfigureTunnelProvider(_) => ErrorStateReason::TunnelProvider,

            #[cfg(target_os = "ios")]
            Self::LocateTunDevice(_) => ErrorStateReason::TunDevice,

            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Self::GetRouteHandle(_) => ErrorStateReason::Internal,

            #[cfg(target_os = "linux")]
            Self::GetDefaultInterface(_) => ErrorStateReason::Internal,
        })
    }
}

impl tunnel::Error {
    fn error_state_reason(&self) -> Option<ErrorStateReason> {
        match self {
            Self::SelectGateways(e) => match e {
                GatewayDirectoryError::SameEntryAndExitGatewayFromCountry { .. } => {
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
            _ => None,
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl fmt::Display for TunnelState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => f.write_str("Disconnected"),
            Self::Connecting { connection_data } => match connection_data {
                Some(data) => match data.tunnel {
                    TunnelConnectionData::Mixnet(ref data) => {
                        write!(
                            f,
                            "Connecting Mixnet tunnel with entry {} and exit {}",
                            data.nym_address.gateway().to_base58_string(),
                            data.exit_ipr.gateway().to_base58_string(),
                        )
                    }
                    TunnelConnectionData::Wireguard(ref data) => {
                        write!(
                            f,
                            "Connecting WireGuard tunnel with entry {} and exit {}",
                            data.entry.endpoint, data.exit.endpoint
                        )
                    }
                },
                None => f.write_str("Connecting"),
            },
            Self::Connected { connection_data } => match connection_data.tunnel {
                TunnelConnectionData::Mixnet(ref data) => {
                    write!(
                        f,
                        "Connected Mixnet tunnel with entry {} and exit {}",
                        data.nym_address.gateway().to_base58_string(),
                        data.exit_ipr.gateway().to_base58_string(),
                    )
                }
                TunnelConnectionData::Wireguard(ref data) => {
                    write!(
                        f,
                        "Connected WireGuard tunnel with entry {} and exit {}",
                        data.entry.endpoint, data.exit.endpoint
                    )
                }
            },
            Self::Disconnecting { after_disconnect } => match after_disconnect {
                ActionAfterDisconnect::Nothing => f.write_str("Disconnecting"),
                ActionAfterDisconnect::Reconnect => f.write_str("Disconnecting to reconnect"),
                ActionAfterDisconnect::Error => f.write_str("Disconnecting because of an error"),
                ActionAfterDisconnect::Offline => {
                    f.write_str("Disconnecting because device is offline")
                }
            },
            Self::Error(reason) => {
                write!(f, "Error state: {:?}", reason)
            }
            Self::Offline { reconnect } => {
                if *reconnect {
                    write!(f, "Offline, auto-connect once back online")
                } else {
                    write!(f, "Offline")
                }
            }
        }
    }
}

impl fmt::Display for MixnetEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bandwidth(event) => write!(f, "{}", event),
            Self::Connection(event) => write!(f, "{}", event),
            Self::ConnectionStatistics(event) => write!(f, "{}", event),
        }
    }
}

impl fmt::Display for ConnectionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ConnectedIpv4 => "Connected with IPv4",
            Self::ConnectedIpv6 => "Connected with IPv6",
            Self::EntryGatewayDown => {
                "Entry gateway appears down - it's not routing our mixnet traffic"
            }
            Self::ExitGatewayDownIpv4 => "Exit gateway (or ipr) appears down - it's not responding to IPv4 traffic",
            Self::ExitGatewayDownIpv6 => "Exit gateway (or ipr) appears down - it's not responding to IPv6 traffic",
            Self::ExitGatewayRoutingErrorIpv4 => "Exit gateway (or ipr) appears to be having issues routing and forwarding our external IPv4 traffic",
            Self::ExitGatewayRoutingErrorIpv6 => "Exit gateway (or ipr) appears to be having issues routing and forwarding our external IPv6 traffic",
        };

        f.write_str(s)
    }
}

impl fmt::Display for BandwidthEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoBandwidth => f.write_str("No bandwidth"),
            Self::RemainingBandwidth(value) => {
                write!(
                    f,
                    "Remaining bandwidth: {}",
                    si_scale::helpers::bibytes2(*value as f64)
                )
            }
        }
    }
}
