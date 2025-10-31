// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    io,
    path::{Path, PathBuf},
};

use sqlx::{
    ConnectOptions,
    sqlite::{SqliteAutoVacuum, SqliteSynchronous},
};
use time::OffsetDateTime;

use crate::{
    keys::wireguard::{
        WireguardKeyStore, WireguardKeys,
        persistence::{is_expired, random_keys, random_timestamp_from},
    },
    types::RawWireguardKeys,
};

#[derive(Debug, Clone)]
pub struct OnDiskKeys {
    connection_pool: sqlx::SqlitePool,
}

#[derive(Debug, thiserror::Error)]
pub enum OnDiskKeysError {
    #[error("unable to create the directory for the database at {}: {source}", provided_path.display())]
    DatabasePathUnableToCreateParentDirectory {
        provided_path: PathBuf,
        source: io::Error,
    },

    #[error("failed to connect to the underlying connection pool: {source}")]
    DatabaseConnectionError {
        #[source]
        source: sqlx::error::Error,
    },

    #[error("Failed to perform database migration: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("Failed to recover assymmetric key: {0}")]
    RecoverAssymmetricKey(#[from] nym_crypto::asymmetric::x25519::KeyRecoveryError),

    #[error("Database experienced an internal error: {0}")]
    InternalDatabaseError(#[from] sqlx::Error),
}

#[async_trait::async_trait]
impl WireguardKeyStore for OnDiskKeys {
    type StorageError = OnDiskKeysError;

    async fn load_or_create_keys(
        &self,
        gateway_id: &str,
    ) -> Result<WireguardKeys, Self::StorageError> {
        let keys = if let Some(raw_keys) = self.get_keys(gateway_id).await?
            && !is_expired(raw_keys.expiration_time)
        {
            WireguardKeys::try_from(raw_keys)?
        } else {
            let keys = random_keys();
            let expiration_time = random_timestamp_from(OffsetDateTime::now_utc());
            let raw_keys = RawWireguardKeys {
                gateway_id_bs58: gateway_id.to_string(),
                entry_private_key_bs58: keys.entry_keypair().private_key().to_base58_string(),
                exit_private_key_bs58: keys.exit_keypair().private_key().to_base58_string(),
                expiration_time,
            };
            self.set_keys(&raw_keys).await?;
            keys
        };
        Ok(keys)
    }

    async fn clear_keys(&self) -> Result<(), Self::StorageError> {
        self.delete_keys().await?;
        Ok(())
    }
}

// all SQL goes here
impl OnDiskKeys {
    #[allow(unused)]
    pub async fn init<P: AsRef<Path>>(database_path: P) -> Result<Self, OnDiskKeysError> {
        // ensure the whole directory structure exists
        if let Some(parent_dir) = database_path.as_ref().parent() {
            std::fs::create_dir_all(parent_dir).map_err(|source| {
                OnDiskKeysError::DatabasePathUnableToCreateParentDirectory {
                    provided_path: database_path.as_ref().to_path_buf(),
                    source,
                }
            })?;
        }

        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .auto_vacuum(SqliteAutoVacuum::Incremental)
            .filename(database_path)
            .create_if_missing(true)
            .disable_statement_logging();

        let connection_pool = sqlx::SqlitePool::connect_with(opts)
            .await
            .map_err(|source| {
                tracing::error!("Failed to connect to SQLx database: {source}");
                OnDiskKeysError::DatabaseConnectionError { source }
            })?;

        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to initialize SQLx database: {err}");
            })?;

        tracing::debug!("Database migration finished!");
        Ok(OnDiskKeys { connection_pool })
    }

    pub(crate) async fn get_keys(
        &self,
        gateway_id: &str,
    ) -> Result<Option<RawWireguardKeys>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM wireguard_gateway_keys WHERE gateway_id_bs58 = ?")
            .bind(gateway_id)
            .fetch_optional(&self.connection_pool)
            .await
    }

    pub(crate) async fn set_keys(&self, keys: &RawWireguardKeys) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT OR IGNORE INTO wireguard_gateway_keys(gateway_id_bs58, entry_private_key_bs58, exit_private_key_bs58, expiration_time) VALUES (?, ?, ?, ?);
                UPDATE wireguard_gateway_keys
                    SET
                        entry_private_key_bs58 = ?,
                        exit_private_key_bs58 = ?,
                        expiration_time = ?
                    WHERE gateway_id_bs58 = ?
            "#,
            keys.gateway_id_bs58,
            keys.entry_private_key_bs58,
            keys.exit_private_key_bs58,
            keys.expiration_time,
            keys.entry_private_key_bs58,
            keys.exit_private_key_bs58,
            keys.expiration_time,
            keys.gateway_id_bs58,
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub(crate) async fn delete_keys(&self) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                DELETE FROM wireguard_gateway_keys;
            "#,
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}
