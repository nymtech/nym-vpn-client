// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use crate::{Result, SecureEnclaveError, SecureEnclaveKey};

use super::SecureEnclaveProvider;

#[derive(Default)]
pub struct AppleSecureEnclaveProvider;

impl SecureEnclaveProvider for AppleSecureEnclaveProvider {
    fn create_key(&self, alias: &str) -> Result<Arc<dyn SecureEnclaveKey>> {
        let key = secure_enclave_rs::SecureEnclaveKey::generate(
            &secure_enclave_rs::SecureEnclaveKeyOptions {
                tag: alias.as_bytes(),
                access_flags: secure_enclave_rs::SecAccessControlFlags::USER_PRESENCE,
                permanent: true,
            },
        )?;

        Ok(Arc::new(OSSecureEnclaveKey::from(key)))
    }

    fn delete_key(&self, alias: &str) -> crate::Result<()> {
        secure_enclave_rs::SecureEnclaveKey::remove_by_tag(alias.as_bytes())?;
        Ok(())
    }

    fn get_key(&self, alias: &str) -> Result<Arc<dyn SecureEnclaveKey>> {
        let key = secure_enclave_rs::SecureEnclaveKey::get(alias.as_bytes())?;

        Ok(Arc::new(OSSecureEnclaveKey::from(key)))
    }
}

struct OSSecureEnclaveKey {
    inner: secure_enclave_rs::SecureEnclaveKey,
}

impl SecureEnclaveKey for OSSecureEnclaveKey {
    fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(self.inner.decrypt(bytes)?)
    }

    fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(self.inner.encrypt(bytes)?)
    }
}

impl From<secure_enclave_rs::SecureEnclaveKey> for OSSecureEnclaveKey {
    fn from(inner: secure_enclave_rs::SecureEnclaveKey) -> Self {
        Self { inner }
    }
}

impl From<secure_enclave_rs::Error> for SecureEnclaveError {
    fn from(error: secure_enclave_rs::Error) -> Self {
        match error {
            secure_enclave_rs::Error::AuthFailed => Self::AuthenticationFailed,
            secure_enclave_rs::Error::NotFound => Self::KeyNotFound,
            other => Self::Other(other.to_string()),
        }
    }
}
