// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::error::StatsStorageError;
use sqlx_pool_guard::SqlitePoolGuard;

#[derive(Clone, Debug)]
pub struct SqliteStatsStorageManager {
    connection_pool: SqlitePoolGuard,
}

impl SqliteStatsStorageManager {
    pub fn new(connection_pool: SqlitePoolGuard) -> Self {
        Self { connection_pool }
    }

    pub(crate) async fn close(&self) {
        self.connection_pool.close().await
    }

    pub async fn load_seed(&self) -> Result<Option<String>, StatsStorageError> {
        Ok(sqlx::query!("SELECT seed FROM seed")
            .fetch_optional(&*self.connection_pool)
            .await?
            .map(|r| r.seed))
    }

    pub async fn set_seed(&self, seed: String) -> Result<(), StatsStorageError> {
        self.remove_seed().await?;
        sqlx::query!("INSERT INTO seed VALUES (?)", seed)
            .execute(&*self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn remove_seed(&self) -> Result<(), StatsStorageError> {
        sqlx::query!("DELETE FROM seed")
            .execute(&*self.connection_pool)
            .await?;
        Ok(())
    }
}
