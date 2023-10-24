// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IOError(#[from] std::io::Error),

    #[error("Invalid WireGuard Key")]
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

    #[error("Oneshot send error")]
    OneshotSendError,

    #[error("{0}")]
    SDKError(#[from] nym_sdk::Error),

    #[error("Recipient is not formatted correctly")]
    RecipientFormattingError,

    #[error("{0}")]
    TunError(#[from] tun::Error),

    #[error("{0}")]
    WireguardConfigError(#[from] talpid_wireguard::config::Error),

    #[error("{0}")]
    ValidatorClientError(#[from] nym_validator_client::ValidatorClientError),

    #[error("Invalid Gateway ID")]
    InvalidGatewayID,

    #[error("{0}")]
    KeyRecoveryError(#[from] nym_crypto::asymmetric::encryption::KeyRecoveryError),

    #[error("{0}")]
    NymNodeApiClientError(#[from] nym_node_requests::api::client::NymNodeApiClientError),

    #[error("Invalid Gateway API response")]
    InvalidGatewayAPIResponse,

    #[error("{0}")]
    WireguardTypesError(#[from] nym_wireguard_types::error::Error),
}
