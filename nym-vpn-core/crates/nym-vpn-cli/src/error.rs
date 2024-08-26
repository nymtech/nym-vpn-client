// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error(transparent)]
    VpnLib(#[from] nym_vpn_lib::Error),

    #[error(transparent)]
    ImportCredential(#[from] nym_vpn_lib::credentials::ImportCredentialError),

    #[error("failed to read credential path: {0}")]
    FailedToReadCredentialPath(#[source] std::io::Error),

    #[error("failed to create credential data path: {0}")]
    FailedToCreateCredentialDataPath(#[source] std::io::Error),

    #[error("identity not formatted correctly")]
    NodeIdentityFormatting,

    #[error("recipient is not formatted correctly")]
    RecipientFormatting,

    #[error("config path not set")]
    ConfigPathNotSet,

    #[error("failed to parse encoded credential data")]
    FailedToParseEncodedCredentialData(#[source] bs58::decode::Error),

    #[error("{0}")]
    VpnRun(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("vpn task stopped unexpectedly")]
    UnexpectedStop,
}

// Result type based on our error type
pub(crate) type Result<T> = std::result::Result<T, Error>;
