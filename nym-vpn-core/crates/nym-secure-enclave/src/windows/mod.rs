// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod dpapi;
mod tpm;

use std::sync::Arc;

use crate::{Result, SecureEnclaveError, SecureEnclaveKey, SecureEnclaveProvider};
use dpapi::DpapiCipher;
use tpm::TpmCipher;

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
                let tpm_key = create_tpm_key(&alias)?;
                Ok(WindowsSecureEnclaveKey::new(&alias, tpm_key))
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
            let tpm_key = match tpm::Provider::open() {
                Ok(provider) => Some(provider.open_key(&alias)?),
                Err(err) => {
                    tracing::warn!("TPM is unavailable, using DPAPI: {err}");
                    None
                }
            };
            Ok(WindowsSecureEnclaveKey::new(&alias, tpm_key))
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

/// Encrypts and decrypts data with a key held by one of the backends.
trait KeyCipher: Send + Sync {
    /// Marks the data encrypted by this cipher.
    fn tag(&self) -> u8;

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
}

/// Encrypts with the TPM key, or with DPAPI when the machine has no usable TPM, and decrypts
/// with the backend that encrypted the data.
pub struct WindowsSecureEnclaveKey {
    cipher: Arc<dyn KeyCipher>,
    /// Decrypts data encrypted with DPAPI before the TPM key was created.
    dpapi: Arc<DpapiCipher>,
}

impl WindowsSecureEnclaveKey {
    /// The `alias` is the DPAPI entropy, binding the data to the key.
    fn new(alias: &str, tpm_key: Option<tpm::Key>) -> Self {
        let dpapi = Arc::new(DpapiCipher::new(alias.as_bytes()));
        let cipher: Arc<dyn KeyCipher> = match tpm_key {
            Some(tpm_key) => Arc::new(TpmCipher::from(tpm_key)),
            None => dpapi.clone(),
        };
        Self { cipher, dpapi }
    }
}

#[async_trait::async_trait]
impl SecureEnclaveKey for WindowsSecureEnclaveKey {
    async fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let cipher = self.cipher.clone();
        let bytes = bytes.to_owned();
        run_blocking(move || {
            let ciphertext = cipher.encrypt(&bytes)?;
            let mut output = Vec::with_capacity(1 + ciphertext.len());
            output.push(cipher.tag());
            output.extend_from_slice(&ciphertext);
            Ok(output)
        })
        .await
    }

    async fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let cipher = self.cipher.clone();
        let dpapi = self.dpapi.clone();
        let bytes = bytes.to_owned();
        run_blocking(move || match bytes.split_first() {
            Some((&tag, data)) if tag == cipher.tag() => cipher.decrypt(data),
            Some((&tag, data)) if tag == dpapi.tag() => dpapi.decrypt(data),
            _ => Err(SecureEnclaveError::Other(
                "unknown encrypted data format".to_owned(),
            )),
        })
        .await
    }
}

/// Creates the key in the TPM. Returns `None` if the TPM cannot be used, so that DPAPI is used
/// instead.
fn create_tpm_key(alias: &str) -> Result<Option<tpm::Key>> {
    match tpm::Provider::open() {
        Ok(provider) => match provider.create_key(alias) {
            Ok(key) => Ok(Some(key)),
            Err(SecureEnclaveError::KeyAlreadyExists) => Err(SecureEnclaveError::KeyAlreadyExists),
            Err(err) => {
                tracing::warn!("Failed to create TPM key, using DPAPI: {err}");
                Ok(None)
            }
        },
        Err(err) => {
            tracing::warn!("TPM is unavailable, using DPAPI: {err}");
            Ok(None)
        }
    }
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
        let cipher = DpapiCipher::new(b"entropy");
        let encrypted = cipher.encrypt(b"secret").unwrap();

        assert_ne!(encrypted, b"secret");
        assert_eq!(cipher.decrypt(&encrypted).unwrap(), b"secret");
        assert!(DpapiCipher::new(b"other").decrypt(&encrypted).is_err());
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
    async fn encrypts_with_tpm_when_available() {
        let provider = WindowsSecureEnclaveProvider;
        let alias = test_alias();
        let expected_tag = if tpm::Provider::open().is_ok() {
            tpm::TPM_TAG
        } else {
            dpapi::DPAPI_TAG
        };

        let key = provider.create_key(&alias, false).await.unwrap();
        let encrypted = key.encrypt(b"secret").await.unwrap();
        provider.delete_key(&alias).await.unwrap();

        assert_eq!(encrypted[0], expected_tag);
    }

    #[tokio::test]
    async fn decrypts_data_encrypted_with_dpapi() {
        let provider = WindowsSecureEnclaveProvider;
        let alias = test_alias();
        let mut encrypted = vec![dpapi::DPAPI_TAG];
        encrypted.extend(
            DpapiCipher::new(alias.as_bytes())
                .encrypt(b"secret")
                .unwrap(),
        );

        let key = provider.create_key(&alias, false).await.unwrap();
        let decrypted = key.decrypt(&encrypted).await;
        provider.delete_key(&alias).await.unwrap();

        assert_eq!(decrypted.unwrap(), b"secret");
    }

    #[tokio::test]
    async fn rejects_user_presence() {
        let provider = WindowsSecureEnclaveProvider;

        assert!(provider.create_key(&test_alias(), true).await.is_err());
    }
}
