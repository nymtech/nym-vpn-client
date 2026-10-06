// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
mod android;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod apple;
#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios"))]
mod secure_enclave;

use std::sync::Arc;

#[cfg(target_os = "android")]
use android::{AndroidKeyStoreProvider, AndroidSecureEnclaveProvider};
#[cfg(any(target_os = "macos", target_os = "ios"))]
use apple::AppleSecureEnclaveProvider;
#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios"))]
pub use secure_enclave::SecureEnclave;

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

pub trait SecureEnclaveKey {
    /// Encrypt data using the key stored in secure enclave
    fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt data using the key stored in secure enclave
    fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>>;
}

pub trait SecureEnclaveProvider {
    /// Create a new key in secure enclave with the given alias
    fn create_key(&self, alias: &str) -> Result<Arc<dyn SecureEnclaveKey>>;

    /// Delete the key with the given alias from secure enclave
    fn delete_key(&self, alias: &str) -> Result<()>;

    /// Check if a key with the given alias exists in secure enclave
    fn has_key(&self, alias: &str) -> Result<bool>;
}

pub trait AuthenticationSession {
    /// Authenticate using biometrics
    fn authenticate(&self) -> Result<()>;
}

#[cfg(target_os = "android")]
uniffi::setup_scaffolding!();
