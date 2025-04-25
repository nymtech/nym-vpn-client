// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum MixnetError {
    #[error("Failed to setup mixnet storage paths")]
    SetupMixnetStoragePaths(#[source] nym_sdk::Error),

    #[error("Failed to create mixnet client with default storage")]
    CreateMixnetClientWithDefaultStorage(#[source] nym_sdk::Error),

    #[error("Failed to build mixnet client")]
    BuildMixnetClient(#[source] nym_sdk::Error),

    #[error("Failed to connect to mixnet")]
    ConnectToMixnet(#[source] nym_sdk::Error),

    #[error("Failed to connect to mixnet entry gateway {gateway_id}")]
    EntryGateway {
        gateway_id: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("invalid credential")]
    InvalidCredential,

    #[error("Failed to serialize message")]
    SerializeMessage(#[from] bincode::Error),

    #[error(transparent)]
    ConnectionMonitorError(#[from] nym_connection_monitor::Error),

    #[error("Failed to bundle packet")]
    BundlePacket(#[source] nym_ip_packet_requests::codec::Error),

    #[error("Failed to create input message")]
    CreateInputMessage(#[source] nym_sdk::Error),

    #[error("Failed to send input message")]
    SendInputMessage(#[source] nym_sdk::Error),
}
