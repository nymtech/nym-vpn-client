// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
};

use nym_pemstore::Passphrase;
pub use nym_sqlcipher::DatabaseKey;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use zeroize::Zeroizing;

use crate::store_ciphers::{StoreCipher, StoreCipherError};

#[derive(Debug, thiserror::Error)]
pub enum DatabaseKeyStorageError {
    #[error("failed to {ctx}: {path}")]
    Io {
        ctx: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to encrypt database key")]
    Encrypt(#[source] StoreCipherError),

    #[error("failed to decrypt database key")]
    Decrypt(#[source] StoreCipherError),

    #[error("invalid database key length: {0}")]
    InvalidKeyLength(usize),
}

pub type Result<T, E = DatabaseKeyStorageError> = std::result::Result<T, E>;

/// Generates a random key for encrypting SQLite databases with SQLCipher.
pub fn generate_database_key() -> DatabaseKey {
    let mut key = Zeroizing::new([0u8; nym_sqlcipher::KEY_LEN]);
    rand::fill(&mut key[..]);
    DatabaseKey::new(*key)
}

/// Derives the passphrase encrypting private key PEM files from the database key.
pub fn key_passphrase(database_key: &DatabaseKey) -> Passphrase {
    Passphrase::new(hex::encode(database_key.as_bytes()))
}

fn database_key_from_bytes(bytes: &[u8]) -> Result<DatabaseKey> {
    let key: &nym_sqlcipher::Key = bytes
        .try_into()
        .map_err(|_| DatabaseKeyStorageError::InvalidKeyLength(bytes.len()))?;
    Ok(DatabaseKey::new(*key))
}

/// Stores [`DatabaseKey`] on disk, encrypted with the given cipher.
pub struct OnDiskDatabaseKeyStorage {
    path: PathBuf,
    cipher: Box<dyn StoreCipher>,
}

impl OnDiskDatabaseKeyStorage {
    pub fn new(path: impl AsRef<Path>, cipher: impl StoreCipher + 'static) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            cipher: Box::new(cipher),
        }
    }

    /// Loads the database key or creates and stores a new one if it does not exist yet.
    pub async fn load_or_create(&self) -> Result<DatabaseKey> {
        match self.load().await? {
            Some(key) => Ok(key),
            None => self.create().await,
        }
    }

    async fn load(&self) -> Result<Option<DatabaseKey>> {
        match tokio::fs::read(&self.path).await {
            Ok(ciphertext) => {
                let plaintext = self
                    .cipher
                    .decrypt(&ciphertext)
                    .await
                    .map(Zeroizing::new)
                    .map_err(DatabaseKeyStorageError::Decrypt)?;
                database_key_from_bytes(&plaintext).map(Some)
            }
            Err(err) if err.kind() == IoErrorKind::NotFound => Ok(None),
            Err(err) => Err(self.io_error("read file", err)),
        }
    }

    async fn create(&self) -> Result<DatabaseKey> {
        tracing::info!("Creating database key: {}", self.path.display());

        let key = generate_database_key();
        let ciphertext = self
            .cipher
            .encrypt(key.as_bytes())
            .await
            .map_err(DatabaseKeyStorageError::Encrypt)?;

        match self.write_new_file(&ciphertext).await {
            Ok(()) => Ok(key),
            Err(err) if err.kind() == IoErrorKind::AlreadyExists => {
                // Another process created the key in the meantime
                self.load()
                    .await?
                    .ok_or_else(|| self.io_error("read file", IoErrorKind::NotFound.into()))
            }
            Err(err) => Err(self.io_error("write file", err)),
        }
    }

    async fn write_new_file(&self, ciphertext: &[u8]) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)
            .await?;

        #[cfg(unix)]
        file.set_permissions(Permissions::from_mode(0o600)).await?;

        file.write_all(ciphertext).await?;
        file.sync_all().await
    }

    fn io_error(&self, ctx: &'static str, source: std::io::Error) -> DatabaseKeyStorageError {
        DatabaseKeyStorageError::Io {
            ctx,
            path: self.path.clone(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;
    use crate::store_ciphers::noop_cipher::NoopStoreCipher;

    #[tokio::test]
    async fn load_or_create_returns_same_key() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("nested/encryption.key");

        let created = OnDiskDatabaseKeyStorage::new(&path, NoopStoreCipher)
            .load_or_create()
            .await
            .unwrap();
        let loaded = OnDiskDatabaseKeyStorage::new(&path, NoopStoreCipher)
            .load_or_create()
            .await
            .unwrap();

        assert_eq!(created.as_bytes(), loaded.as_bytes());
    }

    #[tokio::test]
    async fn load_rejects_invalid_key() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("encryption.key");
        tokio::fs::write(&path, b"short").await.unwrap();

        let result = OnDiskDatabaseKeyStorage::new(&path, NoopStoreCipher)
            .load_or_create()
            .await;

        assert_matches!(result, Err(DatabaseKeyStorageError::InvalidKeyLength(5)));
    }
}
