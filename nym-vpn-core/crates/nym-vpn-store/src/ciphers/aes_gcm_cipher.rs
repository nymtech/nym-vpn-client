// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_store_cipher::{
    ARGON2_SALT_SIZE, Algorithm, EncryptedData, KdfInfo, Params, StoreCipher, Version,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::{Result, StoreCipher as StoreCipherTrait, StoreCipherError};

const MEMORY_COST: u32 = 16 * 1024;
const ITERATIONS: u32 = 3;
const PARALLELISM: u32 = 1;
const OUTPUT_LENGTH: usize = 32;

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Credentials {
    pub passphrase: Vec<u8>,
    pub salt: [u8; ARGON2_SALT_SIZE],
}

impl Credentials {
    pub fn new(passphrase: Vec<u8>) -> Self {
        Self {
            passphrase,
            salt: KdfInfo::random_salt().expect("failed to generate salt"),
        }
    }
}

pub struct AesGcmStoreCipher {
    inner: nym_store_cipher::StoreCipher,
}

impl AesGcmStoreCipher {
    pub fn new(credentials: Credentials) -> Self {
        let mut kdf_salt = [0u8; ARGON2_SALT_SIZE];
        kdf_salt.copy_from_slice(&credentials.salt);

        let kdf_info = KdfInfo::Argon2 {
            params: Params::new(MEMORY_COST, ITERATIONS, PARALLELISM, Some(OUTPUT_LENGTH)).unwrap(),
            algorithm: Algorithm::Argon2id,
            version: Version::V0x13,
            kdf_salt: credentials.salt,
        };

        let inner = StoreCipher::new(&credentials.passphrase, kdf_info)
            .expect("failed to create nym_store_cipher::StoreCipher");

        Self { inner }
    }
}

#[async_trait::async_trait]
impl StoreCipherTrait for AesGcmStoreCipher {
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let encrypted_data: EncryptedData =
            serde_json::from_slice(&data).map_err(StoreCipherError::Deserialize)?;

        Ok(self.inner.decrypt_data(encrypted_data)?)
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let encrypted_data = self.inner.encrypt_data_ref(data)?;

        serde_json::to_vec(&encrypted_data).map_err(StoreCipherError::Serialize)
    }
}

impl From<nym_store_cipher::Error> for StoreCipherError {
    fn from(value: nym_store_cipher::Error) -> Self {
        StoreCipherError::KeyStore(Arc::new(value))
    }
}

// todo: add tests
