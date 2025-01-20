// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

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

    #[error("failed to remove pending credential request storage: {0}")]
    RemoveStorage(#[source] std::io::Error),
}
