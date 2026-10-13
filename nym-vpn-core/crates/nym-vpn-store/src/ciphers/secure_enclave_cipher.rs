// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use futures::future::BoxFuture;
use tokio::sync::OnceCell;

use nym_secure_enclave::{PlatformSecureEnclaveKey, SecureEnclaveError, SecureEnclaveKey};

use super::{Result, StoreCipher, StoreCipherError};

pub type LazySecureEnclaveKeyFn =
    Box<dyn Fn() -> BoxFuture<'static, Result<PlatformSecureEnclaveKey>> + Send + Sync>;

pub struct SecureEnclaveCipher {
    key: OnceCell<PlatformSecureEnclaveKey>,
    key_fn: LazySecureEnclaveKeyFn,
}

impl SecureEnclaveCipher {
    pub fn new<F>(key_fn: F) -> Self
    where
        F: LazySecureEnclaveKeyFn,
    {
        Self {
            key: OnceCell::new(),
            key_fn: Box::new(key_fn),
        }
    }

    async fn key(&self) -> Result<&PlatformSecureEnclaveKey> {
        self.key.get_or_try_init(|| (self.key_fn)()).await
    }
}

#[async_trait::async_trait]
impl StoreCipher for SecureEnclaveCipher {
    async fn decrypt(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(self.key().await?.decrypt(data).await?)
    }

    async fn encrypt(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(self.key().await?.encrypt(data).await?)
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
