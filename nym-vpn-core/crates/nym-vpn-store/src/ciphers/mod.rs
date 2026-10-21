// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod aes_gcm_cipher;
pub mod noop_cipher;
#[cfg(any(target_os = "ios", target_os = "macos", target_os = "android"))]
pub mod secure_enclave_cipher;

use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum StoreCipherError {
    #[error("authentication failed")]
    AuthenticationFailed,

    #[error("access denied until device is unlocked")]
    AccessDeniedUntilDeviceUnlocked,

    #[error("key store is locked")]
    Locked,

    #[error("User cancelled")]
    UserCancelled,

    #[error("key store error")]
    KeyStore(#[source] Arc<dyn std::error::Error + Send + Sync>),

    #[error("deserialize error")]
    Deserialize(#[source] serde_json::Error),

    #[error("serialization error")]
    Serialize(#[source] serde_json::Error),
}

pub type Result<T, E = StoreCipherError> = std::result::Result<T, E>;

#[async_trait::async_trait]
pub trait StoreCipher: Send + Sync {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
}
