// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod account;
#[cfg(any(target_os = "android", target_os = "ios", test))]
mod blocking_tun;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod dns_handler;
mod entry_blame;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod gateway_ext;
mod ipv6_availability;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
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
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

#[cfg(target_os = "android")]
use std::os::fd::{FromRawFd, OwnedFd};

#[cfg(target_os = "android")]
use crate::tunnel_provider::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::OSTunProvider;
#[cfg(target_os = "android")]
use crate::tunnel_state_machine::blocking_tun::{
    BLOCKING_INTERFACE_ADDRS, blocking_tunnel_settings,
};

use crate::adblocker;
#[cfg(not(target_os = "android"))]
use crate::resolver;
#[cfg(not(target_os = "ios"))]
use crate::socks5_proxy::Socks5ProxyManager;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::socks5_proxy::find_proxy_binary;

use crate::{
    GatewayProviderError, UserAgent, bandwidth_monitor::Error as BandwidthMonitorError,
    mixnet::VpnTopologyServiceHandle,
    tunnel_state_machine::tunnel::gateway_provider::GatewayProvider,
};

use hickory_resolver::config::NameServerConfig;
#[cfg(not(target_os = "ios"))]
use hickory_resolver::config::ProtocolConfig;
use nym_bandwidth_controller::{
    error::BandwidthControllerError, requests::BandwidthControllerRequestSender,
};
use nym_config::defaults::{WG_METADATA_PORT, WG_TUN_DEVICE_IP_ADDRESS_V4};
use nym_credentials_interface::TicketType;
use nym_favorites::RecentsManager;
use nym_offline_monitor::ConnectivityHandle;
use nym_registration_client::MixnetClientConfig;
use nym_statistics::StatisticsSender;
use nym_vpn_account_controller::{AccountCommandSender, AccountStateReceiver};
use nym_vpn_api_client::SkewManager;
use nym_vpn_network_config::{DiscoveryRefresherCommand, Network};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_firewall::{Firewall, FirewallArguments, InitialFirewallState};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_gateway_directory::ResolvedConfig;
use nym_gateway_directory::{Config as GatewayDirectoryConfig, GatewayCacheHandle, NodeIdentity};
use nym_vpn_lib_types::{
    AccountControllerErrorStateReason, ActionAfterDisconnect, ConnectionData, EntryPoint,
    ErrorStateReason, EstablishConnectionData, EstablishConnectionState, ExitPoint,
    GatewayIndependence, GatewaySelectionAlgorithmConfig, GeoExclusionSettings,
    SplitTunnelSettings, TunnelEvent, TunnelState, TunnelType,
};

use tunnel::SelectedGateways;
#[cfg(windows)]
use wintun::SetupWintunAdapterError;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use dns_handler::DnsHandlerHandle;
#[cfg(not(target_os = "ios"))]
use nym_socks5_proxy_ipc::ProxyConfig;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
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

    /// Tunnel specific routing table, traffic not marked will be routed via this routing table.
    #[cfg(target_os = "linux")]
    pub table_id: u32,
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
            #[cfg(target_os = "linux")]
            table_id: crate::TUNNEL_TABLE_ID,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TunnelSettings {
    /// Whether to enable support for IPv6.
    pub enable_ipv6: bool,

    /// Type of tunnel.
    pub tunnel_type: TunnelType,

    /// Allow LAN connections outside of tunnel.
    pub allow_lan: bool,

    /// Enable Ad blocking
    pub enable_ad_blocking: bool,

    /// Select residential exit gateways only.
    pub residential_exit: bool,

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

    /// Split tunneling settings.
    pub split_tunnel: SplitTunnelSettings,

    /// Geo exclusion settings.
    pub geo_exclusion_settings: GeoExclusionSettings,

    /// Configuration of the gateway selection algorithm.
    pub gateway_selection_algorithm_config: GatewaySelectionAlgorithmConfig,

    /// Heuristics for what is accepted as independent entry and exit gateways
    pub gateway_independence: GatewayIndependence,
}

impl TunnelSettings {
    /// The tunnel type to be used
    pub fn tunnel_type_used(&self) -> TunnelType {
        self.tunnel_type
    }

    pub fn ticket_types_required(&self, enabled_lp: bool) -> Vec<TicketType> {
        match self.tunnel_type_used() {
            TunnelType::Mixnet => {
                vec![TicketType::V1MixnetEntry]
            }
            TunnelType::Wireguard => {
                let mut types = vec![TicketType::V1WireguardEntry, TicketType::V1WireguardExit];
                if !enabled_lp {
                    // Mixnet registration requires a Mixnet Ticket
                    types.push(TicketType::V1MixnetEntry);
                }
                types
            }
        }
    }

    pub fn resolver_config(&self) -> Vec<NameServerConfig> {
        let defaults = || crate::DEFAULT_DNS_SERVERS_CONFIG.clone();

        let mut config = match self.dns {
            DnsOptions::Default => defaults(),
            DnsOptions::Custom(ref addrs) => {
                if addrs.is_empty() {
                    defaults()
                } else {
                    addrs
                        .iter()
                        .cloned()
                        .map(NameServerConfig::udp_and_tcp)
                        .collect()
                }
            }
        };
        config.retain(|ns| ns.ip.is_ipv4() || (ns.ip.is_ipv6() && self.enable_ipv6));
        config
    }

    /// Returns IP addresses of the DNS servers suitable for Android DNS configuraiton.
    ///
    /// If Private DNS is enabled, these IPs will be probed for DoT before falling back to UDP/TCP.
    #[cfg(target_os = "android")]
    pub fn android_tunnel_dns(&self) -> Vec<IpAddr> {
        let defaults = || {
            let mut seen = HashSet::new();
            crate::DEFAULT_DNS_SERVERS_CONFIG
                .iter()
                .filter(|ns| {
                    // Android only supports TCP/UDP 53 and TLS (853)
                    ns.connections.iter().any(|conn| {
                        matches!(
                            conn.protocol,
                            ProtocolConfig::Tls { .. } | ProtocolConfig::Udp | ProtocolConfig::Tcp
                        )
                    })
                })
                .map(|ns| ns.ip)
                .filter(|ip| seen.insert(*ip))
                .collect()
        };

        let mut addrs = match self.dns {
            DnsOptions::Default => defaults(),
            DnsOptions::Custom(ref addrs) => {
                if addrs.is_empty() {
                    defaults()
                } else {
                    addrs.clone()
                }
            }
        };
        addrs.retain(|v| v.is_ipv4() || (v.is_ipv6() && self.enable_ipv6));
        addrs
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub fn allowed_dns_endpoints(&self) -> Vec<nym_firewall::Endpoint> {
        match self.dns {
            DnsOptions::Custom(ref addrs) => {
                if addrs.is_empty() {
                    self.allowed_default_dns_endpoints()
                } else {
                    addrs
                        .iter()
                        .filter(|ip| ip.is_ipv4() || (ip.is_ipv6() && self.enable_ipv6))
                        .copied()
                        .flat_map(|ip| {
                            // todo: add support for DoH/DoT in custom DNS options
                            [
                                nym_firewall::Endpoint::new(
                                    ip,
                                    53,
                                    nym_firewall::TransportProtocol::Udp,
                                ),
                                nym_firewall::Endpoint::new(
                                    ip,
                                    53,
                                    nym_firewall::TransportProtocol::Tcp,
                                ),
                            ]
                        })
                        .collect()
                }
            }
            DnsOptions::Default => self.allowed_default_dns_endpoints(),
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub fn allowed_default_dns_endpoints(&self) -> Vec<nym_firewall::Endpoint> {
        crate::DEFAULT_DNS_SERVERS_CONFIG
            .iter()
            .filter(|ns| ns.ip.is_ipv4() || (ns.ip.is_ipv6() && self.enable_ipv6))
            .flat_map(|ns| {
                ns.connections.iter().map(|conn| {
                    let proto = match conn.protocol {
                        ProtocolConfig::Udp
                        | ProtocolConfig::H3 { .. }
                        | ProtocolConfig::Quic { .. } => nym_firewall::TransportProtocol::Udp,
                        ProtocolConfig::Tcp
                        | ProtocolConfig::Https { .. }
                        | ProtocolConfig::Tls { .. } => nym_firewall::TransportProtocol::Tcp,
                    };
                    nym_firewall::Endpoint::new(ns.ip, conn.port, proto)
                })
            })
            .collect::<Vec<_>>()
    }

    pub fn bridges_enabled(&self) -> bool {
        matches!(self.tunnel_type_used(), TunnelType::Wireguard)
            && self.wireguard_tunnel_options.enable_bridges
    }

    pub fn diff(&self, other: &Self) -> TunnelSettingsDiff {
        let mut diff = TunnelSettingsDiff::default();

        if self.enable_ipv6 != other.enable_ipv6 {
            diff.add(TunnelSettingsDiffFields::EnableIpv6);
        }
        if self.tunnel_type != other.tunnel_type {
            diff.add(TunnelSettingsDiffFields::TunnelType);
        }
        if self.allow_lan != other.allow_lan {
            diff.add(TunnelSettingsDiffFields::AllowLan);
        }
        if self.enable_ad_blocking != other.enable_ad_blocking {
            diff.add(TunnelSettingsDiffFields::EnableAdBlocking);
        }
        if self.residential_exit != other.residential_exit {
            diff.add(TunnelSettingsDiffFields::ResidentialExit);
        }
        if self.mixnet_tunnel_options != other.mixnet_tunnel_options {
            diff.add(TunnelSettingsDiffFields::MixnetTunnelOptions);
        }
        if self.wireguard_tunnel_options != other.wireguard_tunnel_options {
            diff.add(TunnelSettingsDiffFields::WireguardTunnelOptions);
            // We care about just the QUIC setting changing.
            if self.wireguard_tunnel_options.enable_bridges
                != other.wireguard_tunnel_options.enable_bridges
            {
                diff.add(TunnelSettingsDiffFields::QUIC);
            }
        }
        if self.gateway_performance_options != other.gateway_performance_options {
            diff.add(TunnelSettingsDiffFields::GatewayPerformanceOptions);
            // We care about just the mixnet performance setting changing.
            if self.gateway_performance_options.mixnet_min_performance
                != other.gateway_performance_options.mixnet_min_performance
            {
                diff.add(TunnelSettingsDiffFields::MixnetPerformanceOptions);
            }
        }
        if self.mixnet_client_config != other.mixnet_client_config {
            diff.add(TunnelSettingsDiffFields::MixnetPerformanceOptions);
        }
        if self.entry_point != other.entry_point {
            diff.add(TunnelSettingsDiffFields::EntryPoint);
        }
        if self.exit_point != other.exit_point {
            diff.add(TunnelSettingsDiffFields::ExitPoint);
        }
        if self.dns != other.dns {
            diff.add(TunnelSettingsDiffFields::Dns);
        }
        if self.split_tunnel != other.split_tunnel {
            diff.add(TunnelSettingsDiffFields::SplitTunnel);
        }
        if self.geo_exclusion_settings != other.geo_exclusion_settings {
            diff.add(TunnelSettingsDiffFields::GeoExclusion);
            if self.geo_exclusion_settings.enabled != other.geo_exclusion_settings.enabled {
                diff.add(TunnelSettingsDiffFields::GeoExclusionEnabled);
            }
            if self.geo_exclusion_settings.excluded_countries
                != other.geo_exclusion_settings.excluded_countries
            {
                diff.add(TunnelSettingsDiffFields::GeoExclusionExcludedCountries);
            }
        }
        if self.gateway_selection_algorithm_config.enable_geo_location
            != other.gateway_selection_algorithm_config.enable_geo_location
        {
            diff.add(TunnelSettingsDiffFields::GeoLocationEnabled);
        }
        if self.gateway_selection_algorithm_config.enable_geo_location
            != other.gateway_selection_algorithm_config.enable_geo_location
        {
            diff.add(TunnelSettingsDiffFields::GatewaySelectionAlgorithmConfig);
        }
        if self.gateway_independence != other.gateway_independence {
            diff.add(TunnelSettingsDiffFields::GatewayIndependence);
        }

        diff
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TunnelSettingsDiffFields {
    EnableIpv6 = 0,
    TunnelType,
    AllowLan,
    EnableAdBlocking,
    ResidentialExit,
    MixnetTunnelOptions,
    WireguardTunnelOptions,
    QUIC,
    GatewayPerformanceOptions,
    MixnetPerformanceOptions,
    EntryPoint,
    ExitPoint,
    Dns,
    SplitTunnel,
    GeoExclusion,
    GeoExclusionEnabled,
    GeoExclusionExcludedCountries,
    GeoLocationEnabled,
    GatewaySelectionAlgorithmConfig,
    GatewayIndependence,
}

impl TunnelSettingsDiffFields {
    /// Returns true when change of such setting requires reconnect
    pub fn should_reconnect(&self, tunnel_type: TunnelType) -> bool {
        match self {
            Self::EnableIpv6
            | Self::TunnelType
            | Self::ResidentialExit
            | Self::QUIC
            | Self::EntryPoint
            | Self::ExitPoint
            | Self::GatewayPerformanceOptions
            | Self::Dns
            | Self::GatewaySelectionAlgorithmConfig
            | Self::GatewayIndependence => true,
            Self::EnableAdBlocking => {
                // On android reconnect is necessary due to packet filtering used for adblocking.
                cfg!(target_os = "android")
            }
            Self::AllowLan
            | Self::SplitTunnel
            | Self::GeoExclusion
            | Self::GeoExclusionEnabled
            | Self::GeoExclusionExcludedCountries
            | Self::GeoLocationEnabled => false,
            Self::MixnetTunnelOptions | Self::MixnetPerformanceOptions => {
                tunnel_type == TunnelType::Mixnet
            }
            // As LP is only Wg mode, only reconnect if two-hop mode. This will change in the future
            Self::WireguardTunnelOptions => tunnel_type == TunnelType::Wireguard,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct TunnelSettingsDiff(HashSet<TunnelSettingsDiffFields>);

impl TunnelSettingsDiff {
    pub fn add(&mut self, field: TunnelSettingsDiffFields) {
        self.0.insert(field);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_field_changed(&self, field: &TunnelSettingsDiffFields) -> bool {
        self.0.contains(field)
    }

    pub fn allow_lan_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::AllowLan)
    }

    pub fn enable_ad_blocking_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::EnableAdBlocking)
    }

    pub fn entry_point_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::EntryPoint)
    }

    pub fn exit_point_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::ExitPoint)
    }

    pub fn quic_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::QUIC)
    }

    pub fn split_tunnel_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::SplitTunnel)
    }

    pub fn mixnet_performance_options_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::MixnetPerformanceOptions)
    }

    pub fn geo_exclusion_enabled_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::GeoExclusionEnabled)
    }

    pub fn geo_exclusion_excluded_countries_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::GeoExclusionExcludedCountries)
    }

    pub fn geo_location_enabled_changed(&self) -> bool {
        self.is_field_changed(&TunnelSettingsDiffFields::GeoLocationEnabled)
    }

    // Returns true if changed tunnel settings should lead to tunnel reconnect
    pub fn should_reconnect(&self, tunnel_type: TunnelType) -> bool {
        self.0
            .iter()
            .any(|change| change.should_reconnect(tunnel_type))
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
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum WireguardMultihopMode {
    /// Multihop using two tun devices to nest tunnels.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[default]
    TunTun,

    #[cfg_attr(any(target_os = "ios", target_os = "android"), default)]
    /// Netstack based multihop.
    Netstack,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct WireguardTunnelOptions {
    pub multihop_mode: WireguardMultihopMode,
    pub enable_bridges: bool,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum DnsOptions {
    #[default]
    Default,
    Custom(Vec<IpAddr>),
}

#[derive(Debug)]
pub enum TunnelCommand {
    /// Connect the tunnel.
    Connect,

    /// Disconnect the tunnel.
    Disconnect,

    /// Set new tunnel settings.
    SetTunnelSettings(TunnelSettings),

    /// Block all network access unless tunnel is disconnecting or disconnected
    Block(ErrorStateReason),
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
#[derive(Debug, Clone, Eq, PartialEq)]
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
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(any(target_os = "ios", target_os = "android"), allow(unused))]
pub struct TunnelMetadata {
    interface: String,
    ips: Vec<IpAddr>,
    ipv4_gateway: Option<Ipv4Addr>,
    ipv6_gateway: Option<Ipv6Addr>,
}

impl TunnelMetadata {
    #[cfg(not(target_os = "ios"))]
    fn get_addresses(&self) -> (Option<Ipv4Addr>, Option<Ipv6Addr>) {
        let v4_address = self
            .ips
            .iter()
            .find(|ip| ip.is_ipv4())
            .map(|addr| match addr {
                IpAddr::V4(addr) => *addr,
                _ => unreachable!("unexpected address family"),
            });
        let v6_address = self
            .ips
            .iter()
            .find(|ip| ip.is_ipv6())
            .map(|addr| match addr {
                IpAddr::V6(addr) => *addr,
                _ => unreachable!("unexpected address family"),
            });
        (v4_address, v6_address)
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
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

#[cfg(not(any(target_os = "android", target_os = "ios")))]
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
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    route_handler: RouteHandler,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    firewall: Firewall,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    dns_handler: DnsHandlerHandle,
    connectivity_handle: ConnectivityHandle,
    #[cfg(not(target_os = "android"))]
    filtering_resolver: resolver::ResolverHandle,
    adblocker: adblocker::AdBlocker,
    #[cfg(not(target_os = "ios"))]
    socks5_proxy_manager: Socks5ProxyManager,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    split_tunnel: nym_split_tunnel::SplitTunnelHandle,
    nym_config: NymConfig,
    tunnel_settings: TunnelSettings,
    tunnel_constants: TunnelConstants,
    status_listener_handle: Option<JoinHandle<()>>,
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,
    /// Held FD for the Android blocking / placeholder VPN interface during Connecting / Error / Offline.
    #[cfg(target_os = "android")]
    android_blocking_tun: Option<OwnedFd>,
    /// Previous live TUN kept when blocking install fails mid-reconnect (avoids ISP window).
    #[cfg(target_os = "android")]
    android_tun_hold: Option<tunnel::Tombstone>,
    account_command_tx: AccountCommandSender,
    account_controller_state: AccountStateReceiver,
    bandwidth_command_tx: BandwidthControllerRequestSender,
    skew_manager: SkewManager,
    statistics_event_sender: StatisticsSender,
    #[cfg(target_os = "linux")]
    nm_connectivity_check_enabled: Option<bool>,
    gateway_provider: GatewayProvider<GatewayCacheHandle>,
    /// Tracks pre-handshake connection failures to attribute blame to the entry gateway.
    entry_blame: entry_blame::EntryBlameTracker<NodeIdentity>,
    topology_service: VpnTopologyServiceHandle,
    recents_manager: RecentsManager<GatewayCacheHandle>,
    discovery_refresher_command_tx: mpsc::UnboundedSender<DiscoveryRefresherCommand>,
    user_agent: UserAgent,
    /// API endpoints resolved in connecting state and used for configuring a bypass in error state.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    resolved_api_endpoints: Option<ResolvedConfig>,
    #[cfg(not(target_os = "ios"))]
    shutdown_token: CancellationToken,
}

impl SharedState {
    /// Unpause discovery / account / gateway cache. This is not a device kill-switch; on Android
    /// other apps follow the VpnService interface, not this flag.
    async fn allow_networking(&self) {
        self.discovery_refresher_command_tx
            .send(DiscoveryRefresherCommand::Pause(false))
            .ok();
        self.account_command_tx
            .set_vpn_api_firewall_down()
            .await
            .ok();
        self.gateway_provider.set_gateway_cache_paused(false);
    }

    /// Notify discovery, account controller, geo-location, and gateway cache when network is restricted.
    async fn disallow_networking(&self) {
        self.discovery_refresher_command_tx
            .send(DiscoveryRefresherCommand::Pause(true))
            .ok();
        self.account_command_tx.set_vpn_api_firewall_up().await.ok();
        self.gateway_provider.set_active_geo_location(false).await;
        self.gateway_provider.set_gateway_cache_paused(true);
    }

    /// Establish (or replace) the Android blocking VPN interface and retain its FD.
    #[cfg(target_os = "android")]
    fn install_android_blocking_tun(&mut self) -> std::io::Result<()> {
        let settings = blocking_tunnel_settings(BLOCKING_INTERFACE_ADDRS[0]);
        let raw_fd = self.tun_provider.configure_tunnel(settings)?;
        // Safety: configure_tunnel returns a freshly owned FD from VpnService.Builder.establish().
        let owned = unsafe { OwnedFd::from_raw_fd(raw_fd) };
        self.android_blocking_tun = Some(owned);
        // Blocking interface replaced any prior live TUN; drop retained tombstone if present.
        self.android_tun_hold = None;
        Ok(())
    }

    /// Install blocking TUN when none is held yet. After a live TUN may have replaced the cover,
    /// use `prepare_blocking_cover_before_release` so a stale FD cannot skip reinstall.
    #[cfg(target_os = "android")]
    fn ensure_android_blocking_tun(&mut self) -> std::io::Result<()> {
        if self.android_blocking_tun.is_some() {
            return Ok(());
        }
        self.install_android_blocking_tun()
    }

    /// Install cover if needed, unpause control-plane only when covered, and publish
    /// `TunnelProvider` when the device has no blocking TUN and no held live TUN.
    #[cfg(target_os = "android")]
    async fn apply_android_error_cover(&mut self, requested: ErrorStateReason) -> ErrorStateReason {
        if let Err(err) = self.ensure_android_blocking_tun() {
            nym_common::trace_err_chain!(
                err,
                "failed to install Android blocking TUN in error state"
            );
        }
        let covered = self.android_blocking_tun.is_some() || self.android_tun_hold.is_some();
        if covered {
            self.allow_networking().await;
        }
        blocking_tun::android_error_reason_if_uncovered(requested, covered)
    }

    /// Drop blocking TUN and any retained previous TUN (intentional Disconnect / real tunnel up).
    #[cfg(target_os = "android")]
    fn clear_android_blocking_tun(&mut self) {
        self.android_blocking_tun = None;
        self.android_tun_hold = None;
    }

    /// Install blocking cover, then release the previous TUN. On install failure, keep the previous
    /// TUN in `android_tun_hold` so reconnect/error cannot open an ISP window.
    #[cfg(target_os = "android")]
    fn prepare_blocking_cover_before_release(
        &mut self,
        mut tombstone: Option<tunnel::Tombstone>,
    ) -> std::io::Result<()> {
        match self.install_android_blocking_tun() {
            Ok(()) => {
                drop(tombstone.take());
                Ok(())
            }
            Err(err) => {
                self.android_tun_hold = tombstone;
                Err(err)
            }
        }
    }

    #[cfg(target_os = "linux")]
    pub fn disable_nm_connectivity_check(&mut self) {
        if self.nm_connectivity_check_enabled.is_none()
            && let Ok(nm) = nym_dbus::network_manager::NetworkManager::new()
        {
            self.nm_connectivity_check_enabled = nm.disable_connectivity_check();
        }
    }

    #[cfg(target_os = "linux")]
    pub fn restore_nm_connectivity_check(&mut self) {
        if let Some(true) = self.nm_connectivity_check_enabled.take()
            && let Ok(nm) = nym_dbus::network_manager::NetworkManager::new()
        {
            nm.enable_connectivity_check();
        }
    }

    async fn enable_ad_blocking(&self, enable: bool) {
        if enable {
            self.adblocker.enable().await;
        } else {
            self.adblocker.disable().await;
        }
    }

    /// Set which applications matching the given paths should be excluded from the tunnel
    ///
    /// On Linux paths aren't used to exclude applications from the tunnel.
    ///
    /// Return whether a split tunnel interface was added or removed.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn set_split_tunnel_exclude_paths(
        &mut self,
    ) -> Result<bool, nym_split_tunnel::SplitTunnelErrorCause> {
        let paths = self.tunnel_settings.split_tunnel.effective_app_paths();
        let hybrid_paths = if self.tunnel_settings.geo_exclusion_settings.enabled {
            match find_proxy_binary() {
                Ok(path) => HashSet::from([path]),
                Err(err) => {
                    tracing::error!(
                        "The SOCKS5 Proxy is enabled, but its binary could not be found!: {err:?}"
                    );
                    HashSet::new()
                }
            }
        } else {
            HashSet::new()
        };

        tracing::info!("Updating Split Tunnel exclude paths: {:?}", paths);

        #[cfg(target_os = "macos")]
        let had_interface = self.split_tunnel.interface().await.is_some();

        self.split_tunnel
            .set_exclude_paths(paths, hybrid_paths)
            .await
            .inspect_err(|error| {
                nym_common::trace_err_chain!(error, "failed to set split tunnel paths");
            })
            .map_err(|err| nym_split_tunnel::SplitTunnelErrorCause::from(&err))?;

        #[cfg(target_os = "macos")]
        {
            let has_interface = self.split_tunnel.interface().await.is_some();
            if had_interface != has_interface {
                tracing::info!(
                    "ST interface is {}",
                    if has_interface {
                        "created"
                    } else {
                        "destroyed"
                    }
                );
            }
            Ok(had_interface != has_interface)
        }

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            Ok(false)
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn enable_split_tunnel(
        &mut self,
        metadata: &TunnelMetadata,
    ) -> Result<(), nym_split_tunnel::SplitTunnelErrorCause> {
        use nym_split_tunnel::VpnInterface;

        let (v4_address, v6_address) = metadata.get_addresses();

        let vpn_interface = VpnInterface {
            name: metadata.interface.clone(),
            v4_address,
            v6_address,
        };

        self.split_tunnel
            .set_tunnel(vpn_interface)
            .await
            .inspect_err(|err| {
                nym_common::trace_err_chain!(err, "failed to set VPN interface for split tunnel")
            })
            .map_err(|err| nym_split_tunnel::SplitTunnelErrorCause::from(&err))
    }

    async fn set_tunnel_settings(&mut self, tunnel_settings: TunnelSettings) {
        self.tunnel_settings = tunnel_settings.clone();
        if let Err(err) = self
            .gateway_provider
            .set_tunnel_settings(tunnel_settings)
            .await
        {
            tracing::error!("Could not update gateway provider with new tunnel settings: {err:?}");
        }
    }

    #[cfg(not(target_os = "ios"))]
    async fn start_or_stop_socks5_proxy(&mut self) {
        if self.tunnel_settings.geo_exclusion_settings.enabled {
            self.start_socks5_proxy().await;
        } else {
            self.stop_socks5_proxy().await;
        }
    }

    #[cfg(not(target_os = "ios"))]
    async fn start_socks5_proxy(&mut self) {
        match self.build_proxy_config() {
            Ok(config) => {
                #[cfg(target_os = "android")]
                let socket_protector: nym_socks5_proxy::SocketProtector = {
                    let provider = Arc::clone(&self.tun_provider);
                    Arc::new(move |fd: i32| provider.bypass(fd))
                };
                self.socks5_proxy_manager
                    .start(
                        config,
                        #[cfg(target_os = "android")]
                        socket_protector,
                        self.shutdown_token.clone(),
                    )
                    .await;
            }
            Err(err) => {
                tracing::error!(
                    "SOCKS5 proxy configuration error: {err}; cannot start the proxy process"
                );
            }
        }
    }

    #[cfg(not(target_os = "ios"))]
    async fn stop_socks5_proxy(&mut self) {
        self.socks5_proxy_manager.stop().await;
    }

    #[cfg(not(target_os = "ios"))]
    fn set_socks5_proxy_tunnel_addrs(
        &mut self,
        tunnel_v4_addr: Option<Ipv4Addr>,
        tunnel_v6_addr: Option<Ipv6Addr>,
    ) {
        self.socks5_proxy_manager
            .set_tunnel_addrs(tunnel_v4_addr, tunnel_v6_addr);
    }

    #[cfg(not(target_os = "ios"))]
    fn set_socks5_proxy_excluded_countries(&self) {
        self.socks5_proxy_manager.set_excluded_countries(
            self.tunnel_settings
                .geo_exclusion_settings
                .excluded_countries
                .clone(),
        );
    }

    #[cfg(not(target_os = "ios"))]
    fn build_proxy_config(&self) -> Result<ProxyConfig, String> {
        let listen_port = self.tunnel_settings.geo_exclusion_settings.listen_port;

        // nym-socks5-proxy files are not network-specific so are stored in data_dir, not network_data_dir.
        // However they used to be stored in the network directory, so migrate them if possible.
        let old_data_dir = self
            .nym_config
            .paths
            .network_data_dir
            .join("nym-socks5-proxy");
        let new_data_dir = self.nym_config.paths.data_dir.join("nym-socks5-proxy");
        if old_data_dir.exists() && !new_data_dir.exists() {
            if let Err(err) = std::fs::rename(&old_data_dir, &new_data_dir) {
                tracing::warn!(
                    "Failed to migrate nym-socks5-proxy directory from {} to {}: {err}",
                    old_data_dir.display(),
                    new_data_dir.display()
                );
            } else {
                tracing::info!(
                    "Migrated nym-socks5-proxy directory from {} to {}",
                    old_data_dir.display(),
                    new_data_dir.display()
                );
            }
        }

        if old_data_dir.exists() {
            // Either both new and old exists or we failed to migrate.
            let _ = std::fs::remove_dir(&old_data_dir);
        }

        // The log file will be written to the actual log directory now, so the old log file can be removed.
        let old_log_file = new_data_dir.join("nym-socks5-proxy.log");
        if old_log_file.exists() {
            let _ = std::fs::remove_file(&old_log_file);
        }

        // The nym-socks5-proxy directory must exist in order for ProxyConfig::validate() to succeed.
        if !new_data_dir.exists()
            && let Err(err) = std::fs::create_dir_all(&new_data_dir)
        {
            return Err(format!(
                "Failed to create directory {}: {err}",
                new_data_dir.display()
            ));
        }

        let log_dir = self.nym_config.paths.log_dir.clone();

        let log_level = if cfg!(debug_assertions) {
            "debug"
        } else {
            "info"
        }
        .to_string();

        let excluded_countries = self
            .tunnel_settings
            .geo_exclusion_settings
            .excluded_countries
            .clone();

        let proxy_config = ProxyConfig {
            listen_port,
            data_dir: new_data_dir,
            log_dir,
            log_level,
            excluded_countries,
        };

        proxy_config.validate()?;

        Ok(proxy_config)
    }
}

#[cfg(target_os = "linux")]
pub struct LinuxSplitTunnelConfiguration {
    /// The cgroup2 used for split tunneling.
    /// Traffic from processes in this cgroup2 should be allowed outside the tunnel.
    pub excluded_cgroup2: Option<nym_cgroup::v2::CGroup2>,

    /// The net_cls id of the v1 cgroup used for split tunneling.
    /// This is used as a fallback to [`Self::excluded_cgroup2`] since old kernels don't support cgroups v2.
    pub net_cls: Option<u32>,
}

use crate::paths::NymConfigPaths;

#[derive(Debug, Clone)]
pub struct NymConfig {
    pub paths: NymConfigPaths,
    pub gateway_config: GatewayDirectoryConfig,
    pub network_rx: watch::Receiver<Box<Network>>,
}

pub struct TunnelStateMachine {
    current_state_handler: Box<dyn TunnelStateHandler>,
    shared_state: SharedState,
    command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
    event_sender: mpsc::UnboundedSender<TunnelEvent>,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    dns_handler_task: JoinHandle<()>,
    #[cfg(not(target_os = "android"))]
    dns_handler_shutdown_token: CancellationToken,
    #[cfg(not(target_os = "android"))]
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
        bandwidth_command_tx: BandwidthControllerRequestSender,
        skew_manager: SkewManager,
        statistics_event_sender: StatisticsSender,
        topology_service: VpnTopologyServiceHandle,
        connectivity_handle: ConnectivityHandle,
        discovery_refresher_command_tx: mpsc::UnboundedSender<DiscoveryRefresherCommand>,
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        split_tunnel: nym_split_tunnel::SplitTunnelHandle,
        gateway_provider: GatewayProvider<GatewayCacheHandle>,
        recents_manager: RecentsManager<GatewayCacheHandle>,
        #[cfg(target_os = "linux")] split_tunnel_config: LinuxSplitTunnelConfiguration,
        #[cfg(not(any(target_os = "android", target_os = "ios")))] route_handler: RouteHandler,
        #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
        file_updater_handle: nym_file_updater::FileUpdaterHandle,
        user_agent: UserAgent,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<()>> {
        #[cfg(not(target_os = "android"))]
        let dns_handler_shutdown_token = CancellationToken::new();

        #[cfg(not(target_os = "android"))]
        let (filtering_resolver, filtering_resolver_handle) =
            resolver::LocalResolver::spawn(true, dns_handler_shutdown_token.child_token())
                .await
                .map_err(Error::StartLocalDnsResolver)?;

        let adblocker = create_adblocker(&nym_config, file_updater_handle);
        if tunnel_settings.enable_ad_blocking {
            adblocker.enable().await;
        }

        #[cfg(not(target_os = "android"))]
        {
            let dns_filter = adblocker.get_dns_filter();
            filtering_resolver.set_dns_filter(dns_filter).await;
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let (dns_handler, dns_handler_task) = DnsHandlerHandle::spawn(
            #[cfg(target_os = "linux")]
            &route_handler,
            dns_handler_shutdown_token.child_token(),
        )
        .map_err(Error::CreateDnsHandler)?;

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let firewall = Firewall::from_args(FirewallArguments {
            allow_lan: tunnel_settings.allow_lan,
            initial_state: InitialFirewallState::None,
            #[cfg(target_os = "linux")]
            fwmark: tunnel_constants.fwmark,
            #[cfg(target_os = "linux")]
            table_id: tunnel_constants.table_id,
            #[cfg(target_os = "linux")]
            excluded_cgroup2: split_tunnel_config.excluded_cgroup2,
            #[cfg(target_os = "linux")]
            net_cls: split_tunnel_config.net_cls,
        })
        .map_err(Error::CreateFirewall)?;

        let mut shared_state = SharedState {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_handler,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            firewall,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            dns_handler,
            connectivity_handle,
            #[cfg(not(target_os = "android"))]
            filtering_resolver,
            adblocker,
            #[cfg(not(target_os = "ios"))]
            socks5_proxy_manager: Socks5ProxyManager::new(),
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            split_tunnel,
            nym_config,
            tunnel_settings,
            tunnel_constants,
            status_listener_handle: None,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            tun_provider,
            #[cfg(target_os = "android")]
            android_blocking_tun: None,
            #[cfg(target_os = "android")]
            android_tun_hold: None,
            account_command_tx,
            account_controller_state,
            bandwidth_command_tx,
            skew_manager,
            statistics_event_sender,
            #[cfg(target_os = "linux")]
            nm_connectivity_check_enabled: None,
            gateway_provider,
            entry_blame: entry_blame::EntryBlameTracker::default(),
            topology_service,
            recents_manager,
            discovery_refresher_command_tx,
            user_agent,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            resolved_api_endpoints: None,
            #[cfg(not(target_os = "ios"))]
            shutdown_token: shutdown_token.clone(),
        };

        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Err(err) = shared_state.set_split_tunnel_exclude_paths().await {
            tracing::error!("failed to set initial split tunnel paths: {err:?}");
        }

        #[cfg(not(target_os = "ios"))]
        if shared_state.tunnel_settings.geo_exclusion_settings.enabled {
            shared_state.start_socks5_proxy().await;
        }

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
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            dns_handler_task,
            #[cfg(not(target_os = "android"))]
            dns_handler_shutdown_token,
            #[cfg(not(target_os = "android"))]
            filtering_resolver_handle,
            shutdown_token,
        };

        Ok(tokio::spawn(tunnel_state_machine.run()))
    }

    async fn run(mut self) {
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
                    self.shared_state
                        .statistics_event_sender
                        .report_tunnel_state(state.clone());
                    let _ = self.event_sender.send(TunnelEvent::NewState(state));
                }
                NextTunnelState::SameState(same_state) => {
                    self.current_state_handler = same_state;
                }
                NextTunnelState::Finished => break,
            }
        }

        tracing::debug!("Tunnel state machine is exiting...");

        #[cfg(not(target_os = "ios"))]
        self.shared_state.stop_socks5_proxy().await;

        #[cfg(not(target_os = "android"))]
        {
            self.dns_handler_shutdown_token.cancel();
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            if let Err(e) = self.dns_handler_task.await {
                tracing::error!("Failed to join on dns handler task: {}", e)
            }

            self.shared_state.route_handler.stop().await;
        }

        #[cfg(not(target_os = "android"))]
        if let Err(e) = self.filtering_resolver_handle.await {
            tracing::error!("Failed to join on filtering resolver task: {}", e)
        }

        self.shared_state.adblocker.stop().await;
    }
}

fn create_adblocker(
    nym_config: &NymConfig,
    file_updater_handle: nym_file_updater::FileUpdaterHandle,
) -> adblocker::AdBlocker {
    // Ad-blocker files are not network-specific so are stored in data_dir, not network_data_dir.
    // However they used to be stored in the network directory, so migrate them if possible.
    let old_adblocker_data_dir = nym_config.paths.network_data_dir.join("ad-blocking");
    let new_adblocker_data_dir = nym_config.paths.data_dir.join("ad-blocking");
    if old_adblocker_data_dir.exists() && !new_adblocker_data_dir.exists() {
        if let Err(err) = std::fs::rename(&old_adblocker_data_dir, &new_adblocker_data_dir) {
            tracing::warn!(
                "Failed to migrate ad-blocking directory from {} to {}: {err}",
                old_adblocker_data_dir.display(),
                new_adblocker_data_dir.display()
            );
        } else {
            tracing::info!(
                "Migrated ad-blocking directory from {} to {}",
                old_adblocker_data_dir.display(),
                new_adblocker_data_dir.display()
            );
        }
    }

    if old_adblocker_data_dir.exists() {
        // Either both new and old exists or we failed to migrate.
        let _ = std::fs::remove_dir(&old_adblocker_data_dir);
    }

    adblocker::AdBlocker::new(new_adblocker_data_dir, file_updater_handle)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[error("failed to create a route handler")]
    CreateRouteHandler(#[source] route_handler::Error),

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[error("failed to create a dns handler")]
    CreateDnsHandler(#[source] dns_handler::Error),

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[error("failed to create firewall")]
    CreateFirewall(#[source] nym_firewall::Error),

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[error("failed to set firewall policy")]
    SetFirewallPolicy(#[source] nym_firewall::Error),

    #[error("failed to resolve API hostnames")]
    ResolveApiHostnames(#[source] Box<nym_gateway_directory::Error>),

    #[cfg(not(target_os = "android"))]
    #[error("failed to start local dns resolver")]
    StartLocalDnsResolver(#[source] resolver::Error),

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[error("failed to start split tunnel task")]
    StartSplitTunnelTask(#[source] nym_split_tunnel::Error),

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

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[error("failed to obtain route handle")]
    GetRouteHandle(#[source] route_handler::Error),

    #[error("failed to get tunnel device name")]
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    GetTunDeviceName(#[source] tun::Error),

    #[error("failed to get the interface IP sender")]
    GetInterfaceIpSender,

    #[error("failed to get tunnel device name")]
    #[cfg(any(target_os = "ios", target_os = "android"))]
    GetTunDeviceName(#[source] tun_name::GetTunNameError),

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[error("failed to set tunnel device ipv6 address")]
    SetTunDeviceIpv6Addr(#[source] std::io::Error),

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[error("failed to add routes")]
    AddRoutes(#[source] route_handler::Error),

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    #[error("failed to set dns")]
    SetDns(#[source] dns_handler::Error),

    #[error("tunnel error")]
    Tunnel(#[from] Box<tunnel::Error>),

    #[error(transparent)]
    Account(#[from] account::Error),

    #[error("ipv6 is disabled in the system")]
    Ipv6Unavailable,

    #[error("wireguard key database")]
    WireguardKeyDb(#[source] nym_vpn_store::keys::wireguard::KeysDbError),

    #[error("failed to create gateway directory client")]
    GatewayDirectoryClient(#[source] nym_gateway_directory::Error),

    #[error("failed to create icmp probe")]
    CreateIcmpProbe(#[source] nym_connection_monitor::IcmpProbeError),

    #[error("failed to create tcp probe")]
    CreateTcpProbe(#[source] nym_connection_monitor::TcpProbeError),

    #[error("failed to configure probe due to missing IPv4 interface address")]
    ProbeRequiresIPv4Addr,

    // Temporary until we support `RegistrationResult::Lp()`
    #[error("invalid tunnel type")]
    InvalidTunnelType,

    #[error("gateway provider shut down")]
    GatewayProviderDown,
}

impl Error {
    fn error_state_reason(self) -> Option<ErrorStateReason> {
        Some(match self {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            Self::CreateRouteHandler(_) | Self::CreateDnsHandler(_) | Self::CreateFirewall(_) => {
                None?
            }
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            Self::AddRoutes(_) => ErrorStateReason::SetRouting,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            Self::SetDns(_) => ErrorStateReason::SetDns,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            Self::SetFirewallPolicy(_) => ErrorStateReason::SetFirewallPolicy,
            Self::CreateTunDevice(_) => ErrorStateReason::TunDevice,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            Self::SetTunDeviceIpv6Addr(_) => ErrorStateReason::TunDevice,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            Self::GetTunDeviceName(_) => ErrorStateReason::TunDevice,
            Self::GetInterfaceIpSender => ErrorStateReason::Internal(self.to_string()),
            #[cfg(any(target_os = "ios", target_os = "android"))]
            Self::GetTunDeviceName(_) => ErrorStateReason::TunDevice,
            Self::ResolveApiHostnames(_) => None?,
            #[cfg(not(target_os = "android"))]
            Self::StartLocalDnsResolver(_) => None?,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            Self::StartSplitTunnelTask(_) => None?,
            #[cfg(windows)]
            Self::SetupWintunAdapter(_) => ErrorStateReason::TunDevice,
            Self::Tunnel(e) => e.error_state_reason()?,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            Self::ConfigureTunnelProvider(_) => ErrorStateReason::TunnelProvider,
            #[cfg(target_os = "ios")]
            Self::LocateTunDevice(_) => ErrorStateReason::TunDevice,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            Self::GetRouteHandle(e) => ErrorStateReason::Internal(e.to_string()),
            Self::Account(e) => e.error_state_reason()?,
            Self::Ipv6Unavailable => ErrorStateReason::Ipv6Unavailable,
            Self::WireguardKeyDb(e) => ErrorStateReason::Internal(e.to_string()),
            Self::GatewayDirectoryClient(e) => ErrorStateReason::Internal(e.to_string()),
            Self::CreateIcmpProbe(e) => ErrorStateReason::Internal(e.to_string()),
            Self::CreateTcpProbe(e) => ErrorStateReason::Internal(e.to_string()),
            Self::ProbeRequiresIPv4Addr => ErrorStateReason::Internal(self.to_string()),
            Self::InvalidTunnelType => ErrorStateReason::Internal(self.to_string()),
            Self::GatewayProviderDown => ErrorStateReason::Internal(self.to_string()),
        })
    }
}

impl tunnel::Error {
    fn error_state_reason(self) -> Option<ErrorStateReason> {
        match self {
            Self::SelectGateways(e) => match *e {
                GatewayProviderError::SameEntryAndExitGateway { .. } => {
                    Some(ErrorStateReason::SameEntryAndExitGateway)
                }
                GatewayProviderError::EntryGatewayUnavailable { .. } => {
                    Some(ErrorStateReason::PerformantEntryGatewayUnavailable)
                }
                GatewayProviderError::ExitGatewayUnavailable { .. } => {
                    Some(ErrorStateReason::PerformantExitGatewayUnavailable)
                }
                GatewayProviderError::NeedsRelaxedIndependenceCriteria => {
                    Some(ErrorStateReason::NeedsRelaxedIndependenceCriteria)
                }
                GatewayProviderError::NeedsDeviceLocation => {
                    Some(ErrorStateReason::NeedsDeviceLocation)
                }
                _ => None,
            },
            Self::BandwidthMonitor(BandwidthMonitorError::EntryGateway(error)) => {
                if error.is_no_retry() {
                    Some(ErrorStateReason::CredentialWastedOnEntryGateway)
                } else {
                    None
                }
            }
            Self::BandwidthMonitor(BandwidthMonitorError::ExitGateway(error)) => {
                if error.is_no_retry() {
                    Some(ErrorStateReason::CredentialWastedOnExitGateway)
                } else {
                    None
                }
            }
            Self::BandwidthController(BandwidthControllerError::TicketbookFetchFailed { .. }) => {
                Some(ErrorStateReason::CredentialFetchingFailed)
            },
            Self::BandwidthController(BandwidthControllerError::TicketbooksUnavailable) => {
                Some(ErrorStateReason::NoCredentialAvailable)
            },

            Self::RegistrationClient(e) => match *e {
                nym_registration_client::RegistrationClientError::WireguardEntryRegistrationCredentialSent { .. } => Some(ErrorStateReason::CredentialWastedOnEntryGateway),
                nym_registration_client::RegistrationClientError::WireguardExitRegistrationCredentialSent { .. } => Some(ErrorStateReason::CredentialWastedOnExitGateway),
                _ => None,
            }
            Self::DupFd(_) => Some(ErrorStateReason::Internal(
                "Failed to dup tunnel fd".to_owned(),
            )),
            #[cfg(target_os = "android")]
            Self::CreateDnsFilterProxy(e) => Some(ErrorStateReason::Internal(
                format!("Failed to create DNS filter proxy: {e}")
            )),
            Self::NoIpAddressAnnounced { .. }
            | Self::MixnetClient(_)
            | Self::BandwidthMonitor(_)
            | Self::BandwidthController(_)
            | Self::Wireguard(_)
            | Self::Cancelled
            | Self::Transport(_) => None,
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
                AcError::ErrorState(AccountControllerErrorStateReason::Storage {
                    context,
                    details,
                }) => Some(ErrorStateReason::Internal(format!(
                    "Failed to initialize account storage: {context} {details}",
                ))),
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

impl From<tunnel::Error> for Error {
    fn from(value: tunnel::Error) -> Self {
        Self::Tunnel(Box::new(value))
    }
}

impl From<tunnel::transports::TransportError> for Error {
    fn from(value: tunnel::transports::TransportError) -> Self {
        Self::Tunnel(Box::new(tunnel::Error::Transport(value)))
    }
}

impl From<nym_registration_client::RegistrationClientError> for Error {
    fn from(value: nym_registration_client::RegistrationClientError) -> Self {
        Self::Tunnel(Box::new(tunnel::Error::RegistrationClient(Box::new(value))))
    }
}
