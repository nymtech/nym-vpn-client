// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

pub(crate) use nym_bridges::{
    config::ClientConfig, connection::BridgeConn, error::TransportError, forward::UdpForwarder,
};

/// First WG datagram wait: five handshake attempts at RekeyTimeout (5s).
pub(crate) const INITIAL_FWD_CONNECTION_TIMEOUT: Duration = Duration::from_secs(21);
pub(crate) const BRIDGE_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) fn filter_bridge_params_ipv4_only(mut params: ClientConfig) -> ClientConfig {
    let addresses = match &mut params {
        ClientConfig::QuicPlain(cfg) => &mut cfg.addresses,
        ClientConfig::TlsPlain(cfg) => &mut cfg.addresses,
        ClientConfig::SshPlain(cfg) => &mut cfg.addresses,
    };
    addresses.retain(|addr| addr.is_ipv4());
    params
}
