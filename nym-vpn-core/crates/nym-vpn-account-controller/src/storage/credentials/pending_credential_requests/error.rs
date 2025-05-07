// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PendingCredentialRequestsStorageError {
    #[error("sqlx error")]
    Sqlx(#[from] sqlx::Error),

    #[error("migrate error")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("bincode error")]
    Bincode(#[from] bincode::Error),

    #[error("file permissions error for {path}")]
    FilePermissions {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to remove pending credential request storage")]
    RemoveStorage(#[source] std::io::Error),
}
