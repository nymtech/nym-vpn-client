// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

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

    /// Check if a key with the given alias exists in secure enclave
    fn has_key(&self, alias: String) -> Result<bool>;
}

pub struct AndroidKeyStoreProvider {
    inner: Arc<dyn AndroidSecureEnclaveProvider>,
}

impl AndroidKeyStoreProvider {
    pub fn new(inner: Arc<dyn AndroidSecureEnclaveProvider>) -> Self {
        Self { inner }
    }
}

impl SecureEnclaveProvider for AndroidKeyStoreProvider {
    fn create_key(&self, alias: &str) -> Result<Arc<dyn SecureEnclaveKey>> {
        let key = self.inner.create_key(alias.to_owned())?;
        // let cast_key: Arc<dyn SecureEnclaveKey> = key;
        Ok(Arc::new(AndroidSecureEnclaveKeyBridge::new(key)))
    }

    fn delete_key(&self, alias: &str) -> Result<()> {
        self.inner.delete_key(alias.to_owned())
    }

    fn has_key(&self, alias: &str) -> Result<bool> {
        self.inner.has_key(alias.to_owned())
    }
}

struct AndroidSecureEnclaveKeyBridge {
    inner: Arc<dyn AndroidSecureEnclaveKey>,
}

impl AndroidSecureEnclaveKeyBridge {
    fn new(inner: Arc<dyn AndroidSecureEnclaveKey>) -> Self {
        Self { inner }
    }
}

impl SecureEnclaveKey for AndroidSecureEnclaveKeyBridge {
    fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        self.inner.decrypt(bytes.to_owned())
    }

    fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        self.inner.encrypt(bytes.to_owned())
    }
}
