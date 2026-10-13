// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_secure_enclave::{PlatformSecureEnclaveKey, SecureEnclaveError, SecureEnclaveKey};

use super::{Result, StoreCipher, StoreCipherError};

pub struct SecureEnclaveCipher {
    key: PlatformSecureEnclaveKey,
}

impl SecureEnclaveCipher {
    pub fn new(key: PlatformSecureEnclaveKey) -> Self {
        Self { key }
    }
}

#[async_trait::async_trait]
impl StoreCipher for SecureEnclaveCipher {
    async fn decrypt(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(self.key.decrypt(data).await?)
    }

    async fn encrypt(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(self.key.encrypt(data).await?)
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
