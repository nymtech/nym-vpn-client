// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use tokio::sync::OnceCell;

use super::{Result, StoreCipher, StoreCipherError};
use nym_secure_enclave::{
    PlatformSecureEnclaveKey, SecureEnclave, SecureEnclaveError, SecureEnclaveKey,
};

pub struct SecureEnclaveCipherOptions {
    pub key_alias: String,
    pub user_presence: bool,
}

pub struct SecureEnclaveCipher {
    cached_key: OnceCell<PlatformSecureEnclaveKey>,
    secure_enclave: SecureEnclave,
    options: SecureEnclaveCipherOptions,
}

impl SecureEnclaveCipher {
    pub fn new(secure_enclave: SecureEnclave, options: SecureEnclaveCipherOptions) -> Self {
        Self {
            cached_key: OnceCell::new(),
            secure_enclave,
            options,
        }
    }

    async fn get_key(&self) -> Result<&PlatformSecureEnclaveKey> {
        self.cached_key
            .get_or_try_init(|| async {
                self.secure_enclave
                    .get_or_create_key(&self.options.key_alias, self.options.user_presence)
                    .await
                    .map_err(StoreCipherError::from)
            })
            .await
    }
}

#[async_trait::async_trait]
impl StoreCipher for SecureEnclaveCipher {
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.get_key().await?.decrypt(data).await?)
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.get_key().await?.encrypt(data).await?)
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
