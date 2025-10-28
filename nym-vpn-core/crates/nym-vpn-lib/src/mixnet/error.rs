// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum MixnetError {
    #[error("failed to serialize message")]
    SerializeMessage(#[from] bincode::Error),

    #[error("failed to bundle packet")]
    BundlePacket(#[source] nym_ip_packet_requests::codec::Error),

    #[error("failed to create input message")]
    CreateInputMessage(#[source] Box<nym_sdk::Error>),

    #[error("failed to send input message")]
    SendInputMessage(#[source] Box<nym_sdk::Error>),

    #[error("failed to join on mixnet listener")]
    JoinMixnetListener(#[source] tokio::task::JoinError),

    #[error("failed to create Topology Provider HTTP client")]
    CreateHTTPClient(#[source] nym_vpn_api_client::error::VpnApiClientError),
}
