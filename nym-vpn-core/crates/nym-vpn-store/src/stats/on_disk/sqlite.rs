// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Clone)]
pub struct SqliteStatsStorageManager {
    _connection_pool: sqlx::SqlitePool,
    _migrated: bool,
}

impl SqliteStatsStorageManager {
    pub fn new(_connection_pool: sqlx::SqlitePool) -> Self {
        Self {
            _connection_pool,
            _migrated: false,
        }
    }

    pub async fn migrate(&mut self) -> Result<(), sqlx::Error> {
        // tracing::debug!("Running migrations");
        // sqlx::migrate!("./migrations")
        //     .run(&self.connection_pool)
        //     .await?;

        // self.migrated = true;
        Ok(())
    }
}
