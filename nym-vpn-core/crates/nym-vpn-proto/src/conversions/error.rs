// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::AddrParseError;

use prost::DecodeError;

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Generic error: {0}")]
    Generic(String),

    #[error("No value set for {0}")]
    NoValueSet(&'static str),

    #[error("Failed to decode {0}: {1}")]
    Decode(&'static str, #[source] DecodeError),

    #[error("Failed to convert time {0}: {1}")]
    ConvertTime(&'static str, #[source] time::Error),

    #[error("Failed to parse address {0}: {1}")]
    ParseAddr(&'static str, #[source] AddrParseError),
}

impl ConversionError {
    pub fn generic<T: ToString>(msg: T) -> Self {
        ConversionError::Generic(msg.to_string())
    }
}
