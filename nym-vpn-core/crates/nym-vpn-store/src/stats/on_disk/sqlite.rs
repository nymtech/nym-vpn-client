// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::OnDiskStatsStorageError;

#[derive(Clone)]
pub struct SqliteStatsStorageManager {
    connection_pool: sqlx::SqlitePool,
    migrated: bool,
}

impl SqliteStatsStorageManager {
    pub fn new(connection_pool: sqlx::SqlitePool) -> Self {
        Self {
            connection_pool,
            migrated: false,
        }
    }

    pub async fn migrate(&mut self) -> Result<(), sqlx::Error> {
        tracing::debug!("Running migrations");
        sqlx::migrate!("./migrations")
            .run(&self.connection_pool)
            .await?;

        self.migrated = true;
        Ok(())
    }

    pub async fn load_seed(&self) -> Result<Option<String>, OnDiskStatsStorageError> {
        Ok(sqlx::query!("SELECT seed FROM seed")
            .fetch_optional(&self.connection_pool)
            .await?
            .map(|r| r.seed))
    }

    pub async fn set_seed(&self, seed: String) -> Result<String, OnDiskStatsStorageError> {
        self.remove_seed().await?;
        sqlx::query!("INSERT INTO seed VALUES (?)", seed)
            .execute(&self.connection_pool)
            .await?;
        Ok(seed)
    }

    pub async fn remove_seed(&self) -> Result<(), OnDiskStatsStorageError> {
        sqlx::query!("DELETE FROM seed")
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}
