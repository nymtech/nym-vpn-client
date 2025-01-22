// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};

use time::OffsetDateTime;

// Represents the identity of a gateway
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Gateway {
    pub id: String,
}

impl Gateway {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Gateway> for Gateway {
    fn from(value: nym_gateway_directory::Gateway) -> Self {
        Self::new(value.identity().to_base58_string())
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ConnectionData {
    /// Mixnet entry gateway.
    pub entry_gateway: Gateway,

    /// Mixnet exit gateway.
    pub exit_gateway: Gateway,

    /// When the tunnel was last established.
    /// Set once the tunnel is connected.
    pub connected_at: Option<OffsetDateTime>,

    /// Tunnel connection data.
    pub tunnel: TunnelConnectionData,
}

impl fmt::Debug for ConnectionData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionData")
            .field("entry_gateway", &self.entry_gateway)
            .field("exit_gateway", &self.exit_gateway)
            .field("connected_at", &self.connected_at)
            .field("tunnel", &self.tunnel)
            .finish()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TunnelConnectionData {
    Mixnet(MixnetConnectionData),
    Wireguard(WireguardConnectionData),
}

// Represents a nym-address of the form id.enc@gateway
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NymAddress {
    pub nym_address: String,
}

impl NymAddress {
    pub fn new(nym_address: String) -> Self {
        Self { nym_address }
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
        Self::new(value.to_string())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MixnetConnectionData {
    pub nym_address: NymAddress,
    pub exit_ipr: NymAddress,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WireguardConnectionData {
    pub entry: WireguardNode,
    pub exit: WireguardNode,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WireguardNode {
    pub endpoint: SocketAddr,
    pub public_key: String,
    pub private_ipv4: Ipv4Addr,
    pub private_ipv6: Ipv6Addr,
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_wg_gateway_client::GatewayData> for WireguardNode {
    fn from(value: nym_wg_gateway_client::GatewayData) -> Self {
        Self {
            endpoint: value.endpoint,
            public_key: value.public_key.to_base64(),
            private_ipv4: value.private_ipv4,
            private_ipv6: value.private_ipv6,
        }
    }
}
