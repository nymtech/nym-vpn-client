// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use hkdf::Hkdf;
pub use nym_pemstore::KeySecret;
use nym_pemstore::RawKey;
pub use nym_sqlcipher::DatabaseKey;
use rand::CryptoRng;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::store_ciphers::{StoreCipherError, aes_gcm_cipher::AesGcmStoreCipher};

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod keychain;
pub mod on_disk;

pub const MASTER_KEY_LEN: usize = 32;

const DATABASE_KEY_LABEL: &[u8] = b"net.nymtech.vpn.database-key.v1";
const KEY_SECRET_LABEL: &[u8] = b"net.nymtech.vpn.pem-key-secret.v1";

/// File encrypted with its own key derived from the [`MasterKey`], so that one file's ciphertext
/// can't be swapped in for another's.
#[derive(Debug, Clone, Copy)]
pub enum StoreFile {
    AccountSummary,
    AccountInfo,
}

impl StoreFile {
    fn label(self) -> &'static [u8] {
        match self {
            StoreFile::AccountSummary => b"net.nymtech.vpn.account-summary-key.v1",
            StoreFile::AccountInfo => b"net.nymtech.vpn.account-info-key.v1",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MasterKeyStorageError {
    #[error("failed to {ctx}: {path}")]
    Io {
        ctx: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to encrypt master key")]
    Encrypt(#[source] StoreCipherError),

    #[error("failed to decrypt master key")]
    Decrypt(#[source] StoreCipherError),

    #[error("invalid master key length: {0}")]
    InvalidKeyLength(usize),

    #[error("failed to derive storage keys from the master key")]
    Derive,

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[error("master key keychain error")]
    Keychain(#[source] nym_secure_enclave::KeychainError),

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[error("master key was removed from the keychain while loading it")]
    KeychainKeyRemoved,
}

pub type Result<T, E = MasterKeyStorageError> = std::result::Result<T, E>;

/// Root secret of the on-device storage, from which [`StorageKeys`] are derived.
///
/// The key is kept on the heap and zeroized on drop.
pub struct MasterKey(Box<Zeroizing<[u8; MASTER_KEY_LEN]>>);

impl MasterKey {
    pub fn generate<R: CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let mut key = Box::new(Zeroizing::new([0u8; MASTER_KEY_LEN]));
        rng.fill_bytes(&mut key[..]);
        MasterKey(key)
    }

    pub fn as_bytes(&self) -> &[u8; MASTER_KEY_LEN] {
        &self.0
    }

    /// Derives the keys encrypting the databases and private key files.
    pub fn derive_storage_keys(&self) -> Result<StorageKeys> {
        let database_key = DatabaseKey::from_bytes(&*self.derive_key(DATABASE_KEY_LABEL)?);
        let key_secret = KeySecret::from(RawKey::from_bytes(&*self.derive_key(KEY_SECRET_LABEL)?));

        Ok(StorageKeys {
            database_key,
            key_secret,
        })
    }

    /// Derives the cipher encrypting the given store file.
    pub fn derive_store_cipher(&self, file: StoreFile) -> Result<AesGcmStoreCipher> {
        Ok(AesGcmStoreCipher::new(&*self.derive_key(file.label())?))
    }

    fn derive_key(&self, label: &[u8]) -> Result<Zeroizing<[u8; MASTER_KEY_LEN]>> {
        let mut okm = Zeroizing::new([0u8; MASTER_KEY_LEN]);
        Hkdf::<Sha256>::new(None, &self.0[..])
            .expand(label, &mut okm[..])
            .map_err(|_| MasterKeyStorageError::Derive)?;
        Ok(okm)
    }
}

impl std::fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MasterKey(<redacted>)")
    }
}

/// Copies the key into a heap allocation. Fails unless it's exactly [`MASTER_KEY_LEN`] bytes.
/// The caller stays responsible for wiping its own buffer.
impl TryFrom<&[u8]> for MasterKey {
    type Error = MasterKeyStorageError;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        let mut key = Box::new(Zeroizing::new([0u8; MASTER_KEY_LEN]));
        if bytes.len() == MASTER_KEY_LEN {
            key.copy_from_slice(bytes);
            Ok(MasterKey(key))
        } else {
            Err(MasterKeyStorageError::InvalidKeyLength(bytes.len()))
        }
    }
}

/// Keys derived from the [`MasterKey`].
#[derive(Clone, Debug)]
pub struct StorageKeys {
    /// Key encrypting SQLite databases with SQLCipher.
    pub database_key: DatabaseKey,

    /// Secret encrypting private key PEM files.
    pub key_secret: KeySecret,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store_ciphers::StoreCipher;

    #[test]
    fn derives_distinct_keys() {
        let master_key = MasterKey::try_from(&[0x42; MASTER_KEY_LEN][..]).unwrap();
        let first = master_key.derive_storage_keys().unwrap();
        let second = master_key.derive_storage_keys().unwrap();

        assert_eq!(
            first.database_key.as_bytes(),
            second.database_key.as_bytes()
        );
        assert_ne!(first.database_key.as_bytes(), master_key.as_bytes());
    }

    #[tokio::test]
    async fn store_files_use_distinct_keys() {
        let master_key = MasterKey::generate(&mut rand::rng());
        let summary = master_key
            .derive_store_cipher(StoreFile::AccountSummary)
            .unwrap();
        let account = master_key
            .derive_store_cipher(StoreFile::AccountInfo)
            .unwrap();

        let encrypted = summary.encrypt(b"data").await.unwrap();

        assert!(account.decrypt(&encrypted).await.is_err());
        assert_eq!(
            master_key
                .derive_store_cipher(StoreFile::AccountSummary)
                .unwrap()
                .decrypt(&encrypted)
                .await
                .unwrap(),
            b"data"
        );
    }
}
