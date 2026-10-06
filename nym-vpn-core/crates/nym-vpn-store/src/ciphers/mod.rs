// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod aes_gcm_cipher;
mod noop_cipher;
#[cfg(any(target_os = "ios", target_os = "android"))]
mod secure_enclave_cipher;

use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum StoreCipherError {
    #[error("authentication failed")]
    AuthenticationFailed,

    #[error("key store error")]
    KeyStore(Arc<dyn std::error::Error>),
}

pub type Result<T, E = StoreCipherError> = std::result::Result<T, E>;

/// Marker at the start of every encrypted item. Plaintext JSON, PEM, base58 and bincode never
/// begin with these bytes, so the loader can tell the two formats apart.
pub const MAGIC: &[u8; 7] = b"NYMENC1";

pub trait StoreCipher {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
}

fn is_encrypted(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}
