// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_client_core::error::ClientCoreError;
use nym_ip_packet_requests::{
    response::DynamicConnectFailureReason, response::StaticConnectFailureReason,
};

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

    // We are not returning the underlying talpid_core::firewall:Error error as I ran into issues
    // with the Send marker trait not being implemented when building on Mac. Possibly we can fix
    // this in the future.
    #[error("{0}")]
    FirewallError(String),

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

    #[error("identity not formatted correctly")]
    NodeIdentityFormattingError,

    #[error("recipient is not formatted correctly")]
    RecipientFormattingError,

    #[error("failed setting up local TUN network device: {0}")]
    TunError(#[from] tun2::Error),

    #[error("{0}")]
    WireguardConfigError(#[from] talpid_wireguard::config::Error),

    #[error("{0}")]
    ValidatorClientError(#[from] nym_validator_client::ValidatorClientError),

    #[error(transparent)]
    ExplorerApiError(#[from] nym_explorer_client::ExplorerApiError),

    #[error("failed to fetch location data from explorer-api: {error}")]
    FailedFetchLocationData {
        error: nym_explorer_client::ExplorerApiError,
    },

    #[error("missing Gateway exit information")]
    MissingExitPointInformation,

    #[error("missing Gateway entry information")]
    MissingEntryPointInformation,

    #[error("invalid Gateway ip: {0}")]
    InvalidGatewayIp(String),

    #[error("invalid Gateway address")]
    InvalidGatewayAddress,

    #[error("invalid Gateway location")]
    InvalidGatewayLocation,

    #[error("{0}")]
    KeyRecoveryError(#[from] nym_crypto::asymmetric::encryption::KeyRecoveryError),

    #[error("{0}")]
    NymNodeApiClientError(#[from] nym_node_requests::api::client::NymNodeApiClientError),

    #[error("failed to lookup described gateways: {source}")]
    FailedToLookupDescribedGateways {
        source: nym_validator_client::ValidatorClientError,
    },

    #[error("gateway was requested by location, but we don't have any location data - is the explorer-api set correctly?")]
    RequestedGatewayByLocationWithoutLocationDataAvailable,

    #[error("requested gateway not found in the remote list: {0}")]
    RequestedGatewayIdNotFound(String),

    #[error("invalid Gateway API response")]
    InvalidGatewayAPIResponse,

    #[error("{0}")]
    WireguardTypesError(#[from] nym_wireguard_types::error::Error),

    #[error("could not obtain the default interface")]
    DefaultInterfaceError,

    #[error("could not obtain the LAN gateway from default interface: {0}")]
    DefaultInterfaceGatewayError(String),

    #[error("received response with version v{received}, the client is too new and can only understand v{expected}")]
    ReceivedResponseWithOldVersion { expected: u8, received: u8 },

    #[error("received response with version v{received}, the client is too old and can only understand v{expected}")]
    ReceivedResponseWithNewVersion { expected: u8, received: u8 },

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

    #[error("missing ip packet router address for gateway")]
    MissingIpPacketRouterAddress,

    #[error("no matching gateway found")]
    NoMatchingGateway,

    #[error("no gateway available for location {0}")]
    NoMatchingGatewayForLocation(String),

    #[error("failed to select gateway based on low latency: {source}")]
    FailedToSelectGatewayBasedOnLowLatency { source: ClientCoreError },

    #[error("failed to select gateway randomly")]
    FailedToSelectGatewayRandomly,

    #[error("deadlock when trying to aquire mixnet client mutes")]
    MixnetClientDeadlock,

    #[error("timeout waiting for mixnet client to start")]
    StartMixnetTimeout,

    #[error("vpn could not be started")]
    NotStarted,

    #[error("vpn errored on stop")]
    StopError,

    #[cfg(any(unix, target_os = "android"))]
    #[error("{0}")]
    TunProvider(#[from] talpid_tunnel::tun_provider::Error),

    #[error("{0}")]
    TalpidCoreMpsc(#[from] talpid_core::mpsc::Error),

    #[cfg(target_os = "ios")]
    #[error("{0}")]
    UniffiError(#[from] crate::platform::error::FFIError),
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
