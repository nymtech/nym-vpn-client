// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum MixnetError {
    #[error("failed to setup mixnet storage paths")]
    SetupMixnetStoragePaths(#[source] Box<nym_sdk::Error>),

    #[error("failed to create mixnet client with default storage")]
    CreateMixnetClientWithDefaultStorage(#[source] Box<nym_sdk::Error>),

    #[error("failed to build mixnet client")]
    BuildMixnetClient(#[source] Box<nym_sdk::Error>),

    #[error("failed to connect to mixnet")]
    ConnectToMixnet(#[source] Box<nym_sdk::Error>),

    #[error("failed to connect to mixnet entry gateway {gateway_id}")]
    EntryGateway {
        gateway_id: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("invalid credential")]
    InvalidCredential,

    #[error("failed to serialize message")]
    SerializeMessage(#[from] bincode::Error),

    #[error(transparent)]
    ConnectionMonitorError(#[from] nym_connection_monitor::Error),

    #[error("failed to bundle packet")]
    BundlePacket(#[source] nym_ip_packet_requests::codec::Error),

    #[error("failed to create input message")]
    CreateInputMessage(#[source] Box<nym_sdk::Error>),

    #[error("failed to send input message")]
    SendInputMessage(#[source] Box<nym_sdk::Error>),

    #[error("mixnet client is already disposed")]
    ClientAlreadyDisposed,

    #[error("failed to join on mixnet listener")]
    JoinMixnetListener(#[source] tokio::task::JoinError),
}
