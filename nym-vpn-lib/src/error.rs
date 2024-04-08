// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

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

    #[error("failed setting up local TUN network device: {0}")]
    TunError(#[from] tun2::Error),

    #[error("{0}")]
    WireguardConfigError(#[from] talpid_wireguard::config::Error),

    #[error("recipient is not formatted correctly")]
    RecipientFormattingError,

    #[error("{0}")]
    ValidatorClientError(#[from] nym_validator_client::ValidatorClientError),

    #[error(transparent)]
    ExplorerApiError(#[from] nym_explorer_client::ExplorerApiError),

    #[error("missing Gateway exit information")]
    MissingExitPointInformation,

    #[error("missing Gateway entry information")]
    MissingEntryPointInformation,

    #[error("{0}")]
    KeyRecoveryError(#[from] nym_crypto::asymmetric::encryption::KeyRecoveryError),

    #[error("{0}")]
    NymNodeApiClientError(#[from] nym_node_requests::api::client::NymNodeApiClientError),

    #[error("gateway was requested by location, but we don't have any location data - is the explorer-api set correctly?")]
    RequestedGatewayByLocationWithoutLocationDataAvailable,

    #[error("invalid Gateway API response")]
    InvalidGatewayAPIResponse,

    #[error("{0}")]
    WireguardTypesError(#[from] nym_wireguard_types::error::Error),

    #[error("could not obtain the default interface")]
    DefaultInterfaceError,

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
    #[error("failed to serialize message")]
    FailedToSerializeMessage {
        #[from]
        source: bincode::Error,
    },

    #[error("failed to create icmp echo request packet")]
    IcmpEchoRequestPacketCreationFailure,

    #[error("failed to create icmp packet")]
    IcmpPacketCreationFailure,

    #[error("failed to create ipv4 packet")]
    Ipv4PacketCreationFailure,

    #[error("gateway does not contain a two character country ISO")]
    CountryCodeNotFound,

    #[error("{0}")]
    GatewayDirectoryError(#[from] nym_gateway_directory::Error),

    #[error("failed to import credential: {source}")]
    FailedToImportCredential { source: nym_id::NymIdError },

    #[error("failed decode base58 credential: {source}")]
    FailedToDecodeBase58Credential { source: bs58::decode::Error },

    #[error("config path not set")]
    ConfigPathNotSet,
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
