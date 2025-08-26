// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

use time::OffsetDateTime;

use crate::TunnelType;

// Represents the identity of a gateway
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GatewayId {
    pub id: String,
}

impl GatewayId {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Gateway> for GatewayId {
    fn from(value: nym_gateway_directory::Gateway) -> Self {
        Self::new(value.identity().to_base58_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstablishConnectionData {
    /// Mixnet entry gateway.
    pub entry_gateway: GatewayId,

    /// Mixnet exit gateway.
    pub exit_gateway: GatewayId,

    /// Tunnel connection data.
    /// Becomes available once tunnel connection is initiated.
    pub tunnel: Option<TunnelConnectionData>,
}

/// Describes the current state when establishing connection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EstablishConnectionState {
    /// Resolving API IP addresses
    ResolvingApiAddresses,

    /// Awaiting for account to be ready for establishing connection
    AwaitingAccountReadiness,

    /// Refreshing gateways
    RefreshingGateways,

    /// Selecting gateways
    SelectingGateways,

    /// Connecting mixnet client
    ConnectingMixnetClient,

    /// Establishing tunnel connection
    ConnectingTunnel,
}

impl fmt::Display for EstablishConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EstablishConnectionState::ResolvingApiAddresses => "resolving api addresses",
            EstablishConnectionState::AwaitingAccountReadiness => "awaiting account readiness",
            EstablishConnectionState::RefreshingGateways => "refreshing gateways",
            EstablishConnectionState::SelectingGateways => "selecting gateways",
            EstablishConnectionState::ConnectingMixnetClient => "connecting mixnet client",
            EstablishConnectionState::ConnectingTunnel => "connecting tunnel",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionData {
    /// Mixnet entry gateway.
    pub entry_gateway: GatewayId,

    /// Mixnet exit gateway.
    pub exit_gateway: GatewayId,

    /// When the tunnel connection was last established.
    pub connected_at: OffsetDateTime,

    /// Tunnel connection data.
    pub tunnel: TunnelConnectionData,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TunnelConnectionData {
    Mixnet(MixnetConnectionData),
    Wireguard(WireguardConnectionData),
    WrappedWireguard(WrappedWireguardConnectionData),
}

impl TunnelConnectionData {
    pub fn tunnel_type(&self) -> TunnelType {
        match self {
            TunnelConnectionData::Mixnet(_) => TunnelType::Mixnet,
            TunnelConnectionData::Wireguard(_) => TunnelType::Wireguard,
        }
    }
}

// Represents a nym-address of the form id.enc@gateway
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NymAddress {
    pub nym_address: String,
    pub gateway_id: String,
}

impl NymAddress {
    pub fn new(nym_address: String, gateway_id: String) -> Self {
        Self {
            nym_address,
            gateway_id,
        }
    }

    pub fn gateway_id(&self) -> &str {
        &self.gateway_id
    }
}

impl fmt::Display for NymAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.nym_address.fmt(f)
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Recipient> for NymAddress {
    fn from(value: nym_gateway_directory::Recipient) -> Self {
        Self::new(value.to_string(), value.gateway().to_base58_string())
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::IpPacketRouterAddress> for NymAddress {
    fn from(value: nym_gateway_directory::IpPacketRouterAddress) -> Self {
        NymAddress::from(nym_gateway_directory::Recipient::from(value))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MixnetConnectionData {
    pub nym_address: NymAddress,
    pub exit_ipr: NymAddress,
    pub entry_ip: IpAddr,
    pub exit_ip: IpAddr,
    pub ipv4: Ipv4Addr,
    pub ipv6: Option<Ipv6Addr>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WireguardConnectionData {
    pub entry: WireguardNode,
    pub exit: WireguardNode,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WrappedWireguardConnectionData {
    pub entry_bridge_addr: SocketAddr,
    pub entry: WireguardNode,
    pub exit: WireguardNode,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WireguardNode {
    pub endpoint: SocketAddr,
    pub public_key: String,
    pub private_ipv4: Ipv4Addr,
    pub private_ipv6: Option<Ipv6Addr>,
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_wg_gateway_client::GatewayData> for WireguardNode {
    fn from(value: nym_wg_gateway_client::GatewayData) -> Self {
        Self {
            endpoint: value.endpoint,
            public_key: value.public_key.to_base64(),
            private_ipv4: value.private_ipv4,
            private_ipv6: Some(value.private_ipv6),
        }
    }
}
