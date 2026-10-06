// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_secure_enclave::{SecureEnclaveError, SecureEnclaveKey};

use super::{Result, StoreCipher, StoreCipherError};

pub struct SecureEnclaveCipher {
    key: Arc<dyn SecureEnclaveKey>,
}

impl SecureEnclaveCipher {
    fn new(key: Arc<dyn SecureEnclaveKey>) -> Self {
        Self { key }
    }
}

impl StoreCipher for SecureEnclaveCipher {
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.key.encrypt(data)?)
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.key.decrypt(data)?)
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
