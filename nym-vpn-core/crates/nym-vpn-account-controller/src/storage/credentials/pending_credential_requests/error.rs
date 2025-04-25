// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PendingCredentialRequestsStorageError {
    #[error("Sqlx error")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migrate error")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("Bincode error")]
    Bincode(#[from] bincode::Error),

    #[error("File permissions error for {path}")]
    FilePermissions {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to remove pending credential request storage")]
    RemoveStorage(#[source] std::io::Error),
}
