// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use crate::SecureEnclaveError;

use super::{Result, SecureEnclaveKey, SecureEnclaveProvider};

// Traits that must be implemented in Android app
#[uniffi::export(with_foreign)]
pub trait AndroidSecureEnclaveKey: Send + Sync {
    /// Encrypt data using the key stored in secure enclave
    fn encrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>>;

    /// Decrypt data using the key stored in secure enclave
    fn decrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>>;
}

#[uniffi::export(with_foreign)]
pub trait AndroidSecureEnclaveProvider: Send + Sync {
    /// Create a new key in secure enclave with the given alias
    fn create_key(&self, alias: String) -> Result<Arc<dyn AndroidSecureEnclaveKey>>;

    /// Delete the key with the given alias from secure enclave
    fn delete_key(&self, alias: String) -> Result<()>;

    /// Get object standing in-place of key that can be used to perform encryption or decryption.
    fn get_key(&self, alias: String) -> Result<Arc<dyn AndroidSecureEnclaveKey>>;
}

pub struct AndroidKeyStoreProvider {
    inner: Arc<dyn AndroidSecureEnclaveProvider>,
}

impl AndroidKeyStoreProvider {
    pub fn new(inner: Arc<dyn AndroidSecureEnclaveProvider>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl SecureEnclaveProvider for AndroidKeyStoreProvider {
    type Key = AndroidKeyStoreKey;

    async fn create_key(&self, alias: String) -> Result<Self::Key> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let key = inner.create_key(alias)?;
            Ok(AndroidKeyStoreKey::new(key))
        })
        .await
        .map_err(|_| SecureEnclaveError::Other("thread panic".to_owned()))?
    }

    async fn delete_key(&self, alias: String) -> Result<()> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || inner.delete_key(alias))
            .await
            .map_err(|_| SecureEnclaveError::Other("thread panic".to_owned()))?
    }

    async fn get_key(&self, alias: String) -> Result<Self::Key> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let key = inner.get_key(alias)?;
            Ok(AndroidKeyStoreKey::new(key))
        })
        .await
        .map_err(|_| SecureEnclaveError::Other("thread panic".to_owned()))?
    }
}

#[derive(Clone)]
pub struct AndroidKeyStoreKey {
    inner: Arc<dyn AndroidSecureEnclaveKey>,
}

impl AndroidKeyStoreKey {
    fn new(inner: Arc<dyn AndroidSecureEnclaveKey>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl SecureEnclaveKey for AndroidKeyStoreKey {
    async fn decrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || inner.decrypt(bytes))
            .await
            .map_err(|_| SecureEnclaveError::Other("thread panic".to_owned()))?
    }

    async fn encrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || inner.encrypt(bytes))
            .await
            .map_err(|_| SecureEnclaveError::Other("thread panic".to_owned()))?
    }
}
