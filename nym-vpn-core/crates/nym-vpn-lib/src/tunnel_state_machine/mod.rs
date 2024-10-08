// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
mod default_interface;
mod dns_handler;
//mod firewall_handler;
mod route_handler;
mod states;
mod tun_ipv6;
mod tunnel;

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};

use time::OffsetDateTime;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use nym_gateway_directory::{
    Config as GatewayDirectoryConfig, EntryPoint, ExitPoint, NodeIdentity, Recipient,
};
use nym_ip_packet_requests::IpPair;
use nym_wg_gateway_client::GatewayData;
use nym_wg_go::PublicKey;

use dns_handler::DnsHandler;
//use firewall_handler::FirewallHandler;
use route_handler::RouteHandler;
use states::DisconnectedState;

use crate::MixnetClientConfig;

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
    NewState((Box<dyn TunnelStateHandler>, TunnelState)),
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

    /// Mixnet tunnel options.
    pub mixnet_tunnel_options: MixnetTunnelOptions,

    /// Overrides gateway config.
    pub gateway_performance_options: GatewayPerformanceOptions,

    /// Overrides mixnet client config when provided.
    /// Leave `None` to use sane defaults.
    pub mixnet_client_config: Option<MixnetClientConfig>,

    /// Entry node.
    pub entry_point: EntryPoint,

    /// Exit node.
    pub exit_point: ExitPoint,

    /// DNS configuration.
    pub dns: DnsOptions,
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

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum DnsOptions {
    #[default]
    Default,
    Custom(Vec<IpAddr>),
}

impl Default for TunnelSettings {
    fn default() -> Self {
        Self {
            tunnel_type: TunnelType::Wireguard,
            mixnet_tunnel_options: MixnetTunnelOptions::default(),
            mixnet_client_config: None,
            gateway_performance_options: GatewayPerformanceOptions::default(),
            entry_point: EntryPoint::Random,
            exit_point: ExitPoint::Random,
            dns: DnsOptions::Default,
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

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Record)]
pub struct ConnectionData {
    /// Mixnet entry gateway
    pub entry_gateway: NodeIdentity,

    /// Mixnet exit gateway
    pub exit_gateway: NodeIdentity,

    /// When the tunnel was last established.
    pub connected_at: OffsetDateTime,

    /// Tunnel connection data.
    pub tunnel: TunnelConnectionData,
}

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Enum)]
pub enum TunnelConnectionData {
    Mixnet(MixnetConnectionData),
    Wireguard(WireguardConnectionData),
}

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Record)]
pub struct MixnetConnectionData {
    pub nym_address: Recipient,
    pub exit_ipr: Recipient,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
}

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Record)]
pub struct WireguardNode {
    pub endpoint: SocketAddr,
    pub public_key: PublicKey,
    pub private_ipv4: Ipv4Addr,
}

impl From<GatewayData> for WireguardNode {
    fn from(value: GatewayData) -> Self {
        Self {
            endpoint: value.endpoint,
            public_key: value.public_key,
            private_ipv4: value.private_ipv4,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Record)]
pub struct WireguardConnectionData {
    pub entry: WireguardNode,
    pub exit: WireguardNode,
}

#[derive(Debug, Clone, Eq, PartialEq, uniffi::Enum)]
pub enum TunnelState {
    Disconnected,
    Connecting,
    Connected {
        connection_data: ConnectionData,
    },
    Disconnecting {
        after_disconnect: ActionAfterDisconnect,
    },
    Error(ErrorStateReason),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, uniffi::Enum)]
pub enum ActionAfterDisconnect {
    Nothing,
    Reconnect,
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

    /// Failure to establish mixnet connection.
    EstablishMixnetConnection,

    /// Failure to establish wireguard connection.
    EstablishWireguardConnection,

    /// Tunnel went down at runtime.
    TunnelDown,

    /// Program errors that must not happen.
    Internal,
}

#[derive(Debug, uniffi::Enum)]
pub enum TunnelEvent {
    NewState(TunnelState),
    MixnetEvent(MixnetEvent),
}

#[derive(Debug, Copy, Clone, uniffi::Enum)]
pub enum MixnetEvent {
    Bandwidth(BandwidthEvent),
    Connection(ConnectionEvent),
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

pub struct SharedState {
    mixnet_event_sender: mpsc::UnboundedSender<MixnetEvent>,
    route_handler: RouteHandler,
    //firewall_handler: FirewallHandler,
    dns_handler: DnsHandler,
    nym_config: NymConfig,
    tunnel_settings: TunnelSettings,
    status_listener_handle: Option<JoinHandle<()>>,
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
    shutdown_token: CancellationToken,
}

impl TunnelStateMachine {
    pub async fn spawn(
        command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
        event_sender: mpsc::UnboundedSender<TunnelEvent>,
        nym_config: NymConfig,
        tunnel_settings: TunnelSettings,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<()>> {
        let (current_state_handler, _) = DisconnectedState::enter();

        let route_handler = RouteHandler::new()
            .await
            .map_err(Error::CreateRouteHandler)?;
        let dns_handler = DnsHandler::new(
            #[cfg(target_os = "linux")]
            &route_handler,
        )
        .await
        .map_err(Error::CreateDnsHandler)?;
        //let firewall_handler = FirewallHandler::new().map_err(Error::CreateFirewallHandler)?;

        let (mixnet_event_sender, mixnet_event_receiver) = mpsc::unbounded_channel();

        let shared_state: SharedState = SharedState {
            mixnet_event_sender,
            route_handler,
            //firewall_handler,
            dns_handler,
            nym_config,
            tunnel_settings,
            status_listener_handle: None,
        };

        let tunnel_state_machine = Self {
            current_state_handler,
            shared_state,
            command_receiver,
            event_sender,
            mixnet_event_receiver,
            shutdown_token,
        };

        Ok(tokio::spawn(tunnel_state_machine.run()))
    }

    async fn run(mut self) {
        let mut mixnet_event_receiver = self.mixnet_event_receiver;
        let cloned_event_sender = self.event_sender.clone();
        tokio::spawn(async move {
            while let Some(event) = mixnet_event_receiver.recv().await {
                if let Err(e) = cloned_event_sender.send(TunnelEvent::MixnetEvent(event)) {
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

                    tracing::debug!("New tunnel state: {:?}", new_state);
                    let _ = self.event_sender.send(TunnelEvent::NewState(new_state));
                }
                NextTunnelState::SameState(same_state) => {
                    self.current_state_handler = same_state;
                }
                NextTunnelState::Finished => break,
            }
        }

        tracing::debug!("Tunnel state machine is exiting...");
        self.shared_state.route_handler.stop().await;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create a route handler: {}", _0)]
    CreateRouteHandler(#[source] route_handler::Error),

    #[error("failed to create a dns handler: {}", _0)]
    CreateDnsHandler(#[source] dns_handler::Error),

    //#[error("failed to create firewall handler: {}", _0)]
    //CreateFirewallHandler(#[source] firewall_handler::Error),
    #[error("failed to create tunnel device: {}", _0)]
    CreateTunDevice(#[source] tun::Error),

    #[error("failed to obtain route handle: {}", _0)]
    GetRouteHandle(#[source] route_handler::Error),

    #[cfg(target_os = "linux")]
    #[error("failed to obtain default interface: {}", _0)]
    GetDefaultInterface(String),

    #[error("failed to get tunnel device name")]
    GetTunDeviceName(#[source] tun::Error),

    #[error("failed to set tunnel device ipv6 address")]
    SetTunDeviceIpv6Addr(#[source] std::io::Error),

    #[error("failed to add routes: {}", _0)]
    AddRoutes(#[source] route_handler::Error),

    #[error("failed to set dns: {}", _0)]
    SetDns(#[source] dns_handler::Error),

    #[error("failed to connect mixnet client: {}", _0)]
    ConnectMixnetClient(#[source] tunnel::Error),

    #[error("failed to connect mixnet tunnel: {}", _0)]
    ConnectMixnetTunnel(#[source] tunnel::Error),

    #[error("failed to connect wireguard tunnel: {}", _0)]
    ConnectWireguardTunnel(#[source] tunnel::Error),

    #[error("failed to run wireguard tunnel: {}", _0)]
    RunWireguardTunnel(#[source] tunnel::Error),
}

impl Error {
    fn error_state_reason(&self) -> ErrorStateReason {
        match self {
            Self::CreateRouteHandler(_) | Self::AddRoutes(_) => ErrorStateReason::Routing,
            Self::CreateDnsHandler(_) | Self::SetDns(_) => ErrorStateReason::Dns,
            //Self::CreateFirewallHandler(_) => ErrorStateReason::Firewall,
            Self::CreateTunDevice(_)
            | Self::GetTunDeviceName(_)
            | Self::SetTunDeviceIpv6Addr(_) => ErrorStateReason::TunDevice,
            Self::ConnectWireguardTunnel(_) | Self::RunWireguardTunnel(_) => {
                // todo: add detail
                ErrorStateReason::EstablishWireguardConnection
            }
            Self::ConnectMixnetTunnel(_) | Self::ConnectMixnetClient(_) => {
                // todo: add detail
                ErrorStateReason::EstablishMixnetConnection
            }
            Self::GetRouteHandle(_) => ErrorStateReason::Internal,
            #[cfg(target_os = "linux")]
            Self::GetDefaultInterface(_) => ErrorStateReason::Internal,
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
