// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_pemstore::{KeySecret, RawKey};
pub use nym_sqlcipher::DatabaseKey;
use zeroize::Zeroizing;

use crate::store_ciphers::StoreCipherError;

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod keychain;
pub mod on_disk;

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

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[error("database key keychain error")]
    Keychain(#[source] nym_secure_enclave::KeychainError),

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[error("database key was removed from the keychain while loading it")]
    KeychainKeyRemoved,
}

pub type Result<T, E = DatabaseKeyStorageError> = std::result::Result<T, E>;

/// Generates a random key for encrypting SQLite databases with SQLCipher.
pub fn generate_database_key() -> DatabaseKey {
    let mut key = Zeroizing::new([0u8; nym_sqlcipher::KEY_LEN]);
    rand::fill(&mut key[..]);
    DatabaseKey::new(*key)
}

/// Secret encrypting private key PEM files, using the database key directly as a raw key.
pub fn key_secret(database_key: &DatabaseKey) -> KeySecret {
    RawKey::new(*database_key.as_bytes()).into()
}

fn database_key_from_bytes(bytes: &[u8]) -> Result<DatabaseKey> {
    let key: &nym_sqlcipher::Key = bytes
        .try_into()
        .map_err(|_| DatabaseKeyStorageError::InvalidKeyLength(bytes.len()))?;
    Ok(DatabaseKey::new(*key))
}
