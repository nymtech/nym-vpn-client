// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub use nym_vpn_api_client::response::{BridgeInformation, BridgeParameters, QuicClientOptions};

pub use nym_bridges::{
    config::ClientConfig, connection::BridgeConn, error::TransportError, forward::UdpForwarder,
};

pub mod quic {
    pub use nym_bridges::transport::quic::ClientOptions;
}
