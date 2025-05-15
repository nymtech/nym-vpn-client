// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("generic error: {0}")]
    Generic(String),

    #[error("no value set for {0}")]
    NoValueSet(&'static str),

    #[error("failed to decode {0}")]
    Decode(&'static str, #[source] prost::UnknownEnumValue),

    #[error("failed to convert time {0}")]
    ConvertTime(&'static str, #[source] time::Error),

    #[error("failed to parse address {0}")]
    ParseAddr(&'static str, #[source] std::net::AddrParseError),
}

impl ConversionError {
    pub fn generic<T: ToString>(msg: T) -> Self {
        ConversionError::Generic(msg.to_string())
    }
}
