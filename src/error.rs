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
}
