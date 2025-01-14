// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};

use nym_gateway_directory::{NodeIdentity, Recipient};
use nym_wg_gateway_client::GatewayData;
use nym_wg_go::PublicKey;
use time::OffsetDateTime;

#[derive(Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TunnelConnectionData {
    Mixnet(MixnetConnectionData),
    Wireguard(WireguardConnectionData),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MixnetConnectionData {
    pub nym_address: Box<Recipient>,
    pub exit_ipr: Box<Recipient>,
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
