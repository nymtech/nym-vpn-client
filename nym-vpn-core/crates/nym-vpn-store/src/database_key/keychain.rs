// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_secure_enclave::KeychainItem;

use super::{
    DatabaseKey, DatabaseKeyStorageError, Result, database_key_from_bytes, generate_database_key,
};

/// Stores [`DatabaseKey`] in the keychain.
pub struct KeychainDatabaseKeyStorage {
    item: KeychainItem,
}

impl KeychainDatabaseKeyStorage {
    pub fn new(item: KeychainItem) -> Self {
        Self { item }
    }

    /// Loads the database key or creates and stores a new one if it does not exist yet.
    pub async fn load_or_create(&self) -> Result<DatabaseKey> {
        match self.load().await? {
            Some(key) => Ok(key),
            None => self.create().await,
        }
    }

    async fn load(&self) -> Result<Option<DatabaseKey>> {
        let bytes = self
            .item
            .get_value()
            .await
            .map_err(DatabaseKeyStorageError::Keychain)?;
        bytes
            .map(|bytes| database_key_from_bytes(&bytes))
            .transpose()
    }

    async fn create(&self) -> Result<DatabaseKey> {
        tracing::info!("Creating database key in the keychain");

        let key = generate_database_key();
        let added = self
            .item
            .add(key.as_bytes())
            .map_err(DatabaseKeyStorageError::Keychain)?;
        if added {
            Ok(key)
        } else {
            // Another process created the key in the meantime
            self.load()
                .await?
                .ok_or(DatabaseKeyStorageError::KeychainKeyRemoved)
        }
    }
}
