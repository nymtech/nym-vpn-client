// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use time::{Date, OffsetDateTime};

use super::models::PendingCredentialRequestStored;

#[derive(Clone)]
pub struct SqliteZkNymRequestsStorageManager {
    connection_pool: sqlx::SqlitePool,
}

impl SqliteZkNymRequestsStorageManager {
    pub fn new(connection_pool: sqlx::SqlitePool) -> Self {
        Self { connection_pool }
    }

    pub async fn close(&self) {
        self.connection_pool.close().await
    }

    pub async fn get_pending_requests(
        &self,
    ) -> Result<Vec<PendingCredentialRequestStored>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM pending_zk_nym_requests")
            .fetch_all(&self.connection_pool)
            .await
    }

    pub async fn remove_stale(&self, cutoff: OffsetDateTime) -> Result<(), sqlx::Error> {
        let affected = sqlx::query!(
            "DELETE FROM pending_zk_nym_requests WHERE timestamp < ?",
            cutoff
        )
        .execute(&self.connection_pool)
        .await?
        .rows_affected();
        tracing::info!("Removed {} stale pending requests", affected);
        Ok(())
    }

    pub async fn get_pending_request_by_id(
        &self,
        id: &str,
    ) -> Result<Option<PendingCredentialRequestStored>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM pending_zk_nym_requests WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.connection_pool)
            .await
    }

    pub async fn insert_pending_request(
        &self,
        id: &str,
        expiration_date: Date,
        request_info: &[u8],
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO pending_zk_nym_requests (id, expiration_date, request_info) VALUES (?, ?, ?)",
            id,
            expiration_date,
            request_info,
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn remove_pending_request(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM pending_zk_nym_requests WHERE id = ?", id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}
