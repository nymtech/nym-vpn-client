// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidWireGuardKey(#[from] talpid_types::net::wireguard::InvalidKey),

    #[error("{0}")]
    AddrParseError(#[from] std::net::AddrParseError),
}
