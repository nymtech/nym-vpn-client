// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_secure_enclave::KeychainItem;

use super::{MasterKey, MasterKeyStorageError, Result};

/// Stores [`MasterKey`] in the keychain.
pub struct KeychainMasterKeyStorage {
    item: KeychainItem,
}

impl KeychainMasterKeyStorage {
    pub fn new(item: KeychainItem) -> Self {
        Self { item }
    }

    /// Loads the master key or creates and stores a new one if it does not exist yet.
    pub async fn load_or_create(&self) -> Result<MasterKey> {
        match self.load().await? {
            Some(key) => Ok(key),
            None => self.create().await,
        }
    }

    async fn load(&self) -> Result<Option<MasterKey>> {
        let bytes = self
            .item
            .get_value()
            .await
            .map_err(MasterKeyStorageError::Keychain)?;
        bytes
            .map(|bytes| MasterKey::try_from(&bytes[..]))
            .transpose()
    }

    async fn create(&self) -> Result<MasterKey> {
        tracing::info!("Creating master key in the keychain");

        let key = MasterKey::generate(&mut rand::rng());
        let added = self
            .item
            .add(key.as_bytes())
            .map_err(MasterKeyStorageError::Keychain)?;
        if added {
            Ok(key)
        } else {
            // Another process created the key in the meantime
            self.load()
                .await?
                .ok_or(MasterKeyStorageError::KeychainKeyRemoved)
        }
    }
}
