// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod key_store;
mod keys;
mod persistence;

use std::path::{Path, PathBuf};

pub use key_store::WireguardKeyStore;
pub use keys::WireguardKeys;
use persistence::{
    ephemeral::{EphemeralKeysError, InMemEphemeralKeys},
    on_disk::{OnDiskKeys, OnDiskKeysError},
};

pub const DB_NAME: &str = "wireguard_keys.db";

#[derive(Clone)]
pub enum WireguardKeysDb {
    OnDisk(OnDiskKeys),
    Ephemeral(InMemEphemeralKeys),
}

#[derive(Debug, thiserror::Error)]
pub enum KeysDbError {
    #[error(transparent)]
    OnDisk(#[from] OnDiskKeysError),

    #[error(transparent)]
    Ephemeral(#[from] EphemeralKeysError),
}

impl WireguardKeysDb {
    pub async fn init<P: AsRef<Path>>(database_dir: Option<P>) -> Result<Self, KeysDbError> {
        let db = if let Some(database_dir) = database_dir {
            let database_path = PathBuf::new().join(database_dir).join(DB_NAME);
            WireguardKeysDb::OnDisk(OnDiskKeys::init(database_path).await?)
        } else {
            WireguardKeysDb::Ephemeral(InMemEphemeralKeys::default())
        };
        Ok(db)
    }
}

#[async_trait::async_trait]
impl WireguardKeyStore for WireguardKeysDb {
    type StorageError = KeysDbError;

    async fn load_or_create_keys(
        &self,
        gateway_id: &str,
    ) -> Result<WireguardKeys, Self::StorageError> {
        let ret = match self {
            WireguardKeysDb::OnDisk(on_disk_keys) => {
                on_disk_keys.load_or_create_keys(gateway_id).await?
            }
            WireguardKeysDb::Ephemeral(in_mem_ephemeral_keys) => {
                in_mem_ephemeral_keys
                    .load_or_create_keys(gateway_id)
                    .await?
            }
        };
        Ok(ret)
    }

    async fn clear_keys(&self) -> Result<(), Self::StorageError> {
        match self {
            WireguardKeysDb::OnDisk(on_disk_keys) => on_disk_keys.clear_keys().await?,
            WireguardKeysDb::Ephemeral(in_mem_ephemeral_keys) => {
                in_mem_ephemeral_keys.clear_keys().await?
            }
        };
        Ok(())
    }
}
