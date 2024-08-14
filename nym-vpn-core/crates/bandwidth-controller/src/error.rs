// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use nym_credentials::error::Error as CredentialsError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BandwidthControllerError {
    #[error("There was a credential storage error - {0}")]
    CredentialStorageError(Box<dyn std::error::Error + Send + Sync>),

    #[error("the credential storage does not contain any usable credentials")]
    NoCredentialsAvailable,

    #[error("Credential error - {0}")]
    CredentialError(#[from] CredentialsError),

    #[error("can't handle recovering storage with revision {stored}. {expected} was expected")]
    UnsupportedCredentialStorageRevision { stored: u8, expected: u8 },
}
