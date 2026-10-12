// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_secure_enclave::{SecureEnclaveError, SecureEnclaveKey};

use super::{Result, StoreCipher, StoreCipherError};

pub struct SecureEnclaveCipher<K> {
    key: K,
}

impl<K: SecureEnclaveKey> SecureEnclaveCipher<K> {
    fn new(key: K) -> Self {
        Self { key }
    }
}

#[async_trait::async_trait]
impl<K: SecureEnclaveKey> StoreCipher for SecureEnclaveCipher<K> {
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.key.decrypt(data.to_owned()).await?)
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.key.encrypt(data.to_owned()).await?)
    }
}

impl From<SecureEnclaveError> for StoreCipherError {
    fn from(value: SecureEnclaveError) -> Self {
        match value {
            SecureEnclaveError::AuthenticationFailed => StoreCipherError::AuthenticationFailed,
            err => StoreCipherError::KeyStore(Arc::new(err)),
        }
    }
}
