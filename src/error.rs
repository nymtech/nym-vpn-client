// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid WireGuard Key")]
    InvalidWireGuardKey,

    #[error("{0}")]
    AddrParseError(#[from] std::net::AddrParseError),
}
