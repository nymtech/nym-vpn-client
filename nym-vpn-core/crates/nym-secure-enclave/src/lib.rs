// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
mod android;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod apple;
#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios"))]
mod secure_enclave;

#[cfg(target_os = "android")]
use android::AndroidKeyStoreProvider;
#[cfg(target_os = "android")]
pub use android::{AndroidSecureEnclaveKey, AndroidSecureEnclaveProvider};
#[cfg(any(target_os = "macos", target_os = "ios"))]
use apple::AppleSecureEnclaveProvider;
#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios"))]
pub use secure_enclave::{PlatformSecureEnclaveKey, SecureEnclave};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(target_os = "android", derive(uniffi::Error))]
pub enum SecureEnclaveError {
    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Key is not found")]
    KeyNotFound,

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T, E = SecureEnclaveError> = std::result::Result<T, E>;

#[async_trait::async_trait]
pub trait SecureEnclaveKey: Send + Sync {
    /// Encrypt data using the key stored in secure enclave
    async fn encrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>>;

    /// Decrypt data using the key stored in secure enclave
    async fn decrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>>;
}

#[async_trait::async_trait]
pub trait SecureEnclaveProvider {
    /// Object standing in-place of key stored in secure enclave
    type Key: SecureEnclaveKey;

    /// Create a new key in secure enclave with the given alias
    async fn create_key(&self, alias: String) -> Result<Self::Key>;

    /// Delete the key with the given alias from secure enclave
    async fn delete_key(&self, alias: String) -> Result<()>;

    /// Get object standing in-place of key that can be used to perform encryption or decryption.
    async fn get_key(&self, alias: String) -> Result<Self::Key>;
}

#[cfg(target_os = "android")]
uniffi::setup_scaffolding!();
