// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to serialize message")]
    FailedToSerializeMessage {
        #[from]
        source: bincode::Error,
    },

    #[error("failed to decode attached tickets: {source}")]
    FailedToDecodeAttachedTickets {
        #[from]
        source: nym_credentials::Error,
    },

    #[error("failed to import attached tickets: {source}")]
    TicketsImportFailure {
        #[from]
        source: nym_sdk::Error,
    },
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
