// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_ip_packet_requests::{DynamicConnectFailureReason, StaticConnectFailureReason};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),

    #[error("invalid WireGuard Key")]
    InvalidWireGuardKey,

    #[error("{0}")]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error("{0}")]
    RoutingError(#[from] talpid_routing::Error),

    #[error("{0}")]
    DNSError(#[from] talpid_core::dns::Error),

    #[error("{0}")]
    FirewallError(#[from] talpid_core::firewall::Error),

    #[error("{0}")]
    WireguardError(#[from] talpid_wireguard::Error),

    #[error("{0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("{0}")]
    CanceledError(#[from] futures::channel::oneshot::Canceled),

    #[error("failed to send close message to wireguard tunnel")]
    FailedToSendWireguardTunnelClose,

    #[error("failed to send shutdown message to wireguard tunnel")]
    FailedToSendWireguardShutdown,

    #[error("{0}")]
    SDKError(#[from] nym_sdk::Error),

    #[error("recipient is not formatted correctly")]
    RecipientFormattingError,

    #[error("{0}")]
    TunError(#[from] tun::Error),

    #[error("{0}")]
    WireguardConfigError(#[from] talpid_wireguard::config::Error),

    #[error("{0}")]
    ValidatorClientError(#[from] nym_validator_client::ValidatorClientError),

    #[error("invalid Gateway ID")]
    InvalidGatewayID,

    #[error("{0}")]
    KeyRecoveryError(#[from] nym_crypto::asymmetric::encryption::KeyRecoveryError),

    #[error("{0}")]
    NymNodeApiClientError(#[from] nym_node_requests::api::client::NymNodeApiClientError),

    #[error("invalid Gateway API response")]
    InvalidGatewayAPIResponse,

    #[error("{0}")]
    WireguardTypesError(#[from] nym_wireguard_types::error::Error),

    #[error("could not obtain the default interface")]
    DefaultInterfaceError,

    #[error("could not obtain the LAN gateway from default interface: {0}")]
    DefaultInterfaceGatewayError(String),

    #[error("got reply for connect request, but it appears intended for the wrong address?")]
    GotReplyIntendedForWrongAddress,

    #[error("unexpected connect response")]
    UnexpectedConnectResponse,

    #[error("mixnet client stopped returning responses")]
    NoMixnetMessagesReceived,

    #[error("timeout waiting for connect response")]
    TimeoutWaitingForConnectResponse,

    #[error("connect request denied: {reason}")]
    StaticConnectRequestDenied { reason: StaticConnectFailureReason },

    #[error("connect request denied: {reason}")]
    DynamicConnectRequestDenied { reason: DynamicConnectFailureReason },
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
