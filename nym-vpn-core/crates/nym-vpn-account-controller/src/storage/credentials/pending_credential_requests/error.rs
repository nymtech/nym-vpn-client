// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use bincode::Options;
use nym_compact_ecash::VerificationKeyAuth;
use nym_credential_storage::persistent_storage::PersistentStorage as PersistentCredentialStorage;
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
    IssuedTicketBook,
};
use nym_credentials_interface::{
    AnnotatedCoinIndexSignature, AnnotatedExpirationDateSignature, RequestInfo, TicketType,
};
use nym_sdk::mixnet::{CredentialStorage, StoragePaths};
use nym_vpn_api_client::types::{Device, VpnApiAccount};
use nym_vpn_store::{mnemonic::Mnemonic, VpnStorage};
use serde::{Deserialize, Serialize};
use sqlx::{ConnectOptions, FromRow};
use time::Date;
use tracing::log::LevelFilter;

#[derive(Debug, thiserror::Error)]
pub enum PendingCredentialRequestsStorageError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migrate error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("file permissions error for {path:?}: {source}")]
    FilePermissions {
        path: PathBuf,
        source: std::io::Error,
    },
}

