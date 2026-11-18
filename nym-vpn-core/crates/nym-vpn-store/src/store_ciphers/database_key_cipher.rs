// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use nym_sqlcipher::DatabaseKey;

use super::{Result, StoreCipher, StoreCipherError};

const VERSION: u8 = 1;
const NONCE_LEN: usize = 12;

/// Encrypts data with AES-256-GCM, using the database key as the cipher key.
///
/// The output is the format version, followed by a random nonce and the ciphertext.
pub struct DatabaseKeyCipher {
    cipher: Aes256Gcm,
}

impl DatabaseKeyCipher {
    pub fn new(database_key: &DatabaseKey) -> Self {
        Self {
            cipher: Aes256Gcm::new(database_key.as_bytes().into()),
        }
    }
}

#[async_trait::async_trait]
impl StoreCipher for DatabaseKeyCipher {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let nonce: [u8; NONCE_LEN] = rand::random();
        let ciphertext = self
            .cipher
            .encrypt(&Nonce::from(nonce), data)
            .map_err(|_| StoreCipherError::Encrypt)?;

        let mut output = Vec::with_capacity(1 + NONCE_LEN + ciphertext.len());
        output.push(VERSION);
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let Some((&VERSION, data)) = data.split_first() else {
            return Err(StoreCipherError::InvalidData);
        };
        let (nonce, ciphertext) = data
            .split_first_chunk::<NONCE_LEN>()
            .ok_or(StoreCipherError::InvalidData)?;

        self.cipher
            .decrypt(&Nonce::from(*nonce), ciphertext)
            .map_err(|_| StoreCipherError::Decrypt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cipher(key: u8) -> DatabaseKeyCipher {
        DatabaseKeyCipher::new(&DatabaseKey::from_bytes(&[key; nym_sqlcipher::KEY_LEN]))
    }

    #[tokio::test]
    async fn round_trips() {
        let cipher = cipher(1);

        let encrypted = cipher.encrypt(b"summary").await.unwrap();

        assert_ne!(&encrypted[1 + NONCE_LEN..], b"summary");
        assert_eq!(cipher.decrypt(&encrypted).await.unwrap(), b"summary");
    }

    #[tokio::test]
    async fn uses_fresh_nonce() {
        let cipher = cipher(1);

        assert_ne!(
            cipher.encrypt(b"summary").await.unwrap(),
            cipher.encrypt(b"summary").await.unwrap()
        );
    }

    #[tokio::test]
    async fn fails_with_different_key() {
        let encrypted = cipher(1).encrypt(b"summary").await.unwrap();

        assert!(matches!(
            cipher(2).decrypt(&encrypted).await,
            Err(StoreCipherError::Decrypt)
        ));
    }

    #[tokio::test]
    async fn rejects_tampered_data() {
        let cipher = cipher(1);
        let mut encrypted = cipher.encrypt(b"summary").await.unwrap();
        *encrypted.last_mut().unwrap() ^= 1;

        assert!(matches!(
            cipher.decrypt(&encrypted).await,
            Err(StoreCipherError::Decrypt)
        ));
    }

    #[tokio::test]
    async fn rejects_unknown_version_and_truncated_data() {
        let cipher = cipher(1);
        let mut encrypted = cipher.encrypt(b"summary").await.unwrap();

        assert!(matches!(
            cipher.decrypt(&encrypted[..NONCE_LEN]).await,
            Err(StoreCipherError::InvalidData)
        ));
        encrypted[0] = VERSION + 1;
        assert!(matches!(
            cipher.decrypt(&encrypted).await,
            Err(StoreCipherError::InvalidData)
        ));
    }
}
