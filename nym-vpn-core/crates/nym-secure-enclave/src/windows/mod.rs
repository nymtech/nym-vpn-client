// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod dpapi;
mod tpm;

use crate::{Result, SecureEnclaveError, SecureEnclaveKey, SecureEnclaveProvider};

/// Marks data encrypted with a key wrapped by the TPM.
const TPM_TAG: u8 = 1;
/// Marks data encrypted with DPAPI.
const DPAPI_TAG: u8 = 2;

/// Stores keys in the TPM, falling back to DPAPI when the machine has no usable TPM.
///
/// Keys belong to the current user, which is SYSTEM for the daemon. User presence is not
/// supported, so [`SecureEnclaveProvider::unlock`] and [`SecureEnclaveProvider::lock`] do nothing.
#[derive(Default)]
pub struct WindowsSecureEnclaveProvider;

#[async_trait::async_trait]
impl SecureEnclaveProvider for WindowsSecureEnclaveProvider {
    type Key = WindowsSecureEnclaveKey;

    async fn create_key(&self, alias: &str, user_presence: bool) -> Result<Self::Key> {
        if user_presence {
            Err(SecureEnclaveError::Other(
                "user presence is not supported on Windows".to_owned(),
            ))
        } else {
            let alias = alias.to_owned();
            run_blocking(move || {
                let backend = create_backend(&alias)?;
                Ok(WindowsSecureEnclaveKey { alias, backend })
            })
            .await
        }
    }

    async fn delete_key(&self, alias: &str) -> Result<()> {
        let alias = alias.to_owned();
        run_blocking(move || match tpm::Provider::open() {
            Ok(provider) => provider.open_key(&alias)?.delete(),
            Err(_) => Ok(()),
        })
        .await
    }

    async fn get_key(&self, alias: &str) -> Result<Self::Key> {
        let alias = alias.to_owned();
        run_blocking(move || {
            let backend = match tpm::Provider::open() {
                Ok(provider) => provider.open_key(&alias).map(|_| Backend::Tpm)?,
                Err(err) => {
                    tracing::warn!("TPM is unavailable, using DPAPI: {err}");
                    Backend::Dpapi
                }
            };
            Ok(WindowsSecureEnclaveKey { alias, backend })
        })
        .await
    }

    async fn unlock(&self, _reason: &str) -> Result<()> {
        Ok(())
    }

    async fn lock(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Backend {
    Tpm,
    Dpapi,
}

/// Encrypts with the backend the key was created with, and decrypts with the backend that
/// encrypted the data.
pub struct WindowsSecureEnclaveKey {
    alias: String,
    backend: Backend,
}

#[async_trait::async_trait]
impl SecureEnclaveKey for WindowsSecureEnclaveKey {
    async fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let alias = self.alias.clone();
        let backend = self.backend;
        let bytes = bytes.to_owned();
        run_blocking(move || {
            let (tag, ciphertext) = match backend {
                Backend::Tpm => (TPM_TAG, open_tpm_key(&alias)?.encrypt(&bytes)?),
                Backend::Dpapi => (DPAPI_TAG, dpapi::protect(&bytes, alias.as_bytes())?),
            };
            let mut output = Vec::with_capacity(1 + ciphertext.len());
            output.push(tag);
            output.extend_from_slice(&ciphertext);
            Ok(output)
        })
        .await
    }

    async fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let alias = self.alias.clone();
        let bytes = bytes.to_owned();
        run_blocking(move || match bytes.split_first() {
            Some((&TPM_TAG, data)) => open_tpm_key(&alias)?.decrypt(data),
            Some((&DPAPI_TAG, data)) => dpapi::unprotect(data, alias.as_bytes()),
            _ => Err(SecureEnclaveError::Other(
                "unknown encrypted data format".to_owned(),
            )),
        })
        .await
    }
}

/// Creates the key in the TPM, or falls back to DPAPI if the TPM cannot be used.
fn create_backend(alias: &str) -> Result<Backend> {
    match tpm::Provider::open() {
        Ok(provider) => match provider.create_key(alias) {
            Ok(_) => Ok(Backend::Tpm),
            Err(SecureEnclaveError::KeyAlreadyExists) => Err(SecureEnclaveError::KeyAlreadyExists),
            Err(err) => {
                tracing::warn!("Failed to create TPM key, using DPAPI: {err}");
                Ok(Backend::Dpapi)
            }
        },
        Err(err) => {
            tracing::warn!("TPM is unavailable, using DPAPI: {err}");
            Ok(Backend::Dpapi)
        }
    }
}

fn open_tpm_key(alias: &str) -> Result<tpm::Key> {
    tpm::Provider::open()
        .map_err(|err| SecureEnclaveError::Other(format!("failed to open TPM: {err}")))?
        .open_key(alias)
}

async fn run_blocking<T, F>(f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|err| SecureEnclaveError::Other(format!("failed to join blocking task: {err}")))?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_alias() -> String {
        format!("net.nymtech.vpn.test-{:016x}", rand::random::<u64>())
    }

    #[test]
    fn dpapi_round_trips() {
        let encrypted = dpapi::protect(b"secret", b"entropy").unwrap();

        assert_ne!(encrypted, b"secret");
        assert_eq!(dpapi::unprotect(&encrypted, b"entropy").unwrap(), b"secret");
        assert!(dpapi::unprotect(&encrypted, b"other").is_err());
    }

    #[tokio::test]
    async fn key_round_trips() {
        let provider = WindowsSecureEnclaveProvider;
        let alias = test_alias();
        let data = vec![0x5a; 1024];

        let key = provider.create_key(&alias, false).await.unwrap();
        let encrypted = key.encrypt(&data).await.unwrap();
        let decrypted = provider
            .get_key(&alias)
            .await
            .unwrap()
            .decrypt(&encrypted)
            .await
            .unwrap();
        provider.delete_key(&alias).await.unwrap();

        assert_eq!(decrypted, data);
    }

    #[tokio::test]
    async fn rejects_user_presence() {
        let provider = WindowsSecureEnclaveProvider;

        assert!(provider.create_key(&test_alias(), true).await.is_err());
    }
}
