// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("generic error: {0}")]
    Generic(String),
}

impl ConversionError {
    pub fn generic<T: ToString>(msg: T) -> Self {
        ConversionError::Generic(msg.to_string())
    }
}
