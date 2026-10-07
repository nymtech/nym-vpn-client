// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

pub fn open_encrypted_sqlite_store(
    path: &Path,
    options: SqliteConnectOptions,
    passphrase: &str,
) -> sqlx::Result<SqlitePool> {
}
