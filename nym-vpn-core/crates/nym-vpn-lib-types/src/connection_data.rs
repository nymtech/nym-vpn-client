// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use crate::TunnelType;

// Represents the identity of a gateway
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
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

// Represents a gateway's info for ConnectionData
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct GatewayLightInfo {
    pub id: String,
    pub country_code: Option<String>,
}

impl GatewayLightInfo {
    pub fn new(id: String, country_code: Option<String>) -> Self {
        Self { id, country_code }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Gateway> for GatewayLightInfo {
    fn from(value: nym_gateway_directory::Gateway) -> Self {
        Self::new(
            value.identity().to_base58_string(),
            value.location.map(|l| l.two_letter_iso_country_code),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct EstablishConnectionData {
    /// Mixnet entry gateway.
    pub entry_gateway: GatewayLightInfo,

    /// Mixnet exit gateway.
    pub exit_gateway: GatewayLightInfo,

    /// Tunnel connection data.
    /// Becomes available once tunnel connection is initiated.
    pub tunnel: Option<TunnelConnectionData>,
}

/// Describes the current state when establishing connection.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum EstablishConnectionState {
    /// Resolving API IP addresses
    ResolvingApiAddresses,

    /// Awaiting for account to be ready for establishing connection
    AwaitingAccountReadiness,

    /// Awaiting for credentials to be ready for establishing connection
    AwaitingCredentialsAvailability,

    /// Refreshing gateways
    RefreshingGateways,

    /// Selecting gateways
    SelectingGateways,

    /// Registering with gateways
    RegisteringWithGateways,

    /// Establishing tunnel connection
    ConnectingTunnel,
}

impl fmt::Display for EstablishConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EstablishConnectionState::ResolvingApiAddresses => "resolving api addresses",
            EstablishConnectionState::AwaitingAccountReadiness => "awaiting account readiness",
            EstablishConnectionState::AwaitingCredentialsAvailability => {
                "awaiting credentials availability"
            }
            EstablishConnectionState::RefreshingGateways => "refreshing gateways",
            EstablishConnectionState::SelectingGateways => "selecting gateways",
            EstablishConnectionState::RegisteringWithGateways => "registering with gateways",
            EstablishConnectionState::ConnectingTunnel => "connecting tunnel",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct ConnectionData {
    /// Mixnet entry gateway.
    pub entry_gateway: GatewayLightInfo,

    /// Mixnet exit gateway.
    pub exit_gateway: GatewayLightInfo,

    /// When the tunnel connection was last established.
    #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601"))]
    pub connected_at: OffsetDateTime,

    /// Tunnel connection data.
    pub tunnel: TunnelConnectionData,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum TunnelConnectionData {
    Mixnet(MixnetConnectionData),
    Wireguard(WireguardConnectionData),
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
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
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
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct MixnetConnectionData {
    pub nym_address: NymAddress,
    pub exit_ipr: NymAddress,
    pub entry_ip: IpAddr,
    pub exit_ip: IpAddr,
    pub ipv4: Ipv4Addr,
    pub ipv6: Option<Ipv6Addr>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct BridgeAddress {
    /// Local forwarder listen address
    pub listen_addr: SocketAddr,

    /// Remote bridge address
    pub remote_addr: SocketAddr,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct WireguardConnectionData {
    pub entry_bridge_addr: Option<BridgeAddress>,
    pub entry: WireguardNode,
    pub exit: WireguardNode,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct WireguardNode {
    pub endpoint: SocketAddr,
    pub public_key: String,
    pub private_ipv4: Ipv4Addr,
    pub private_ipv6: Option<Ipv6Addr>,
}

#[cfg(feature = "nym-type-conversions")]
impl From<&nym_registration_common::WireguardConfiguration> for WireguardNode {
    fn from(value: &nym_registration_common::WireguardConfiguration) -> Self {
        Self {
            endpoint: value.endpoint,
            public_key: value.public_key.to_base64(),
            private_ipv4: value.private_ipv4,
            private_ipv6: Some(value.private_ipv6),
        }
    }
}
