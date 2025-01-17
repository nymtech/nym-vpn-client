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

use crate::{error::Error, AvailableTicketbooks};

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
