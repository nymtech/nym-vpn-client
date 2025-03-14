// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum MixnetError {
    #[error("failed to setup mixnet storage paths: {0}")]
    FailedToSetupMixnetStoragePaths(#[source] nym_sdk::Error),

    #[error("failed to create mixnet client with default storage: {0}")]
    FailedToCreateMixnetClientWithDefaultStorage(#[source] nym_sdk::Error),

    #[error("failed to build mixnet client: {0}")]
    FailedToBuildMixnetClient(#[source] nym_sdk::Error),

    #[error("failed to connect to mixnet: {0}")]
    FailedToConnectToMixnet(#[source] nym_sdk::Error),

    #[error("failed to connect to mixnet entry gateway {gateway_id}: {source}")]
    EntryGateway {
        gateway_id: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("invalid credential")]
    InvalidCredential,

    #[error("failed to serialize message")]
    FailedToSerializeMessage {
        #[from]
        source: bincode::Error,
    },

    #[error("{0}")]
    ConnectionMonitorError(#[from] nym_connection_monitor::Error),

    #[error("failed to bundle packet: {source}")]
    FailedToBundlePacket {
        source: nym_ip_packet_requests::codec::Error,
    },

    #[error("failed to create input message: {source}")]
    FailedToCreateInputMessage { source: nym_sdk::Error },

    #[error("failed to send input message: {source}")]
    FailedToSendInputMessage { source: nym_sdk::Error },
}
