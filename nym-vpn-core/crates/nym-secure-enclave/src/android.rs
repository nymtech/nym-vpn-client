// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use crate::SecureEnclaveError;

use super::{Result, SecureEnclaveKey, SecureEnclaveProvider};

/*

#[uniffi::export(with_foreign)]
pub trait AndroidSecureEnclaveKey: Send + Sync {
    // Pass the memory location pointer and length over FFI
    fn encrypt_raw(&self, ptr_address: u64, len: u32) -> Result<Vec<u8>>;
    fn decrypt_raw(&self, ptr_address: u64, len: u32) -> Result<Vec<u8>>;
}

import java.nio.ByteBuffer

class MyAndroidKey : AndroidSecureEnclaveKey {
    override fun encryptRaw(ptrAddress: Long, len: Int): ByteArray {
        // Create a direct byte buffer targeting the raw native address without copying
        val byteBuffer = UniFfiDirectBuffer(ptrAddress, len)
        val bytes = ByteArray(len)
        byteBuffer.get(bytes)

        // Execute enclave logic...
        return encryptWithSecureEnclave(bytes)
    }

    override fun decryptRaw(ptrAddress: Long, len: Int): ByteArray { ... }
}

// Helper block to map direct memory address to a readable ByteBuffer
fun UniFfiDirectBuffer(address: Long, capacity: Int): ByteBuffer {
    val clazz = Class.forName("java.nio.DirectByteBuffer")
    val constructor = clazz.getDeclaredConstructor(Long::class.javaPrimitiveType, Int::class.javaPrimitiveType)
    constructor.isAccessible = true
    return constructor.newInstance(address, capacity) as ByteBuffer
}

 */

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

    async fn create_key(&self, alias: &str) -> Result<Self::Key> {
        let inner = self.inner.clone();
        let alias = alias.to_owned();
        tokio::task::spawn_blocking(move || {
            let key = inner.create_key(alias)?;
            Ok(AndroidKeyStoreKey::new(key))
        })
        .await
        .map_err(|_| SecureEnclaveError::Other("thread panic".to_owned()))?
    }

    async fn delete_key(&self, alias: &str) -> Result<()> {
        let inner = self.inner.clone();
        let alias = alias.to_owned();
        tokio::task::spawn_blocking(move || inner.delete_key(alias))
            .await
            .map_err(|_| SecureEnclaveError::Other("thread panic".to_owned()))?
    }

    async fn get_key(&self, alias: &str) -> Result<Self::Key> {
        let inner = self.inner.clone();
        let alias = alias.to_owned();
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
    async fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let inner = self.inner.clone();
        let bytes = bytes.to_vec();
        tokio::task::spawn_blocking(move || inner.decrypt(bytes))
            .await
            .map_err(|_| SecureEnclaveError::Other("thread panic".to_owned()))?
    }

    async fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let inner = self.inner.clone();
        let bytes = bytes.to_vec();
        tokio::task::spawn_blocking(move || inner.encrypt(bytes))
            .await
            .map_err(|_| SecureEnclaveError::Other("thread panic".to_owned()))?
    }
}
