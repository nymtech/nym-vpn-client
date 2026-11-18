// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod credentials;

use credentials::SystemdCredentials;

use crate::{Result, SecureEnclaveError, SecureEnclaveKey, SecureEnclaveProvider};

/// Marks data encrypted with `systemd-creds`.
const CREDENTIAL_TAG: u8 = 1;
/// Marks data stored unencrypted, when `systemd-creds` is unavailable.
const PLAINTEXT_TAG: u8 = 2;

/// Encrypts data with `systemd-creds`, which binds it to the TPM and to the host key held by
/// systemd, or to the host key alone if the machine has no TPM. Data is stored unencrypted on
/// systems without the `systemd-creds` service, relying on the file permissions instead.
///
/// There are no keys to manage: the alias of a key authenticates the data it encrypts. Decrypting
/// requires root. User presence is not supported, so [`SecureEnclaveProvider::unlock`] and
/// [`SecureEnclaveProvider::lock`] do nothing.
#[derive(Default)]
pub struct LinuxSecureEnclaveProvider;

#[async_trait::async_trait]
impl SecureEnclaveProvider for LinuxSecureEnclaveProvider {
    type Key = LinuxSecureEnclaveKey;

    async fn create_key(&self, alias: &str, user_presence: bool) -> Result<Self::Key> {
        if user_presence {
            Err(SecureEnclaveError::Other(
                "user presence is not supported on Linux".to_owned(),
            ))
        } else {
            Ok(LinuxSecureEnclaveKey::new(alias))
        }
    }

    async fn delete_key(&self, _alias: &str) -> Result<()> {
        Ok(())
    }

    async fn get_key(&self, alias: &str) -> Result<Self::Key> {
        Ok(LinuxSecureEnclaveKey::new(alias))
    }

    async fn unlock(&self, _reason: &str) -> Result<()> {
        Ok(())
    }

    async fn lock(&self) -> Result<()> {
        Ok(())
    }
}

/// Encrypts with `systemd-creds` when available, and decrypts data stored either way.
pub struct LinuxSecureEnclaveKey {
    /// Name of the credentials, authenticated on decryption.
    name: String,
    credentials: Option<SystemdCredentials>,
}

impl LinuxSecureEnclaveKey {
    fn new(alias: &str) -> Self {
        let credentials = SystemdCredentials::new();
        if credentials.is_none() {
            tracing::warn!("systemd-creds is unavailable, storing data unencrypted");
        }
        Self {
            name: alias.to_owned(),
            credentials,
        }
    }
}

#[async_trait::async_trait]
impl SecureEnclaveKey for LinuxSecureEnclaveKey {
    async fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let (tag, data) = match &self.credentials {
            Some(credentials) => (
                CREDENTIAL_TAG,
                credentials.encrypt(&self.name, bytes).await?,
            ),
            None => (PLAINTEXT_TAG, bytes.to_owned()),
        };
        let mut output = Vec::with_capacity(1 + data.len());
        output.push(tag);
        output.extend_from_slice(&data);
        Ok(output)
    }

    async fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        match bytes.split_first() {
            Some((&CREDENTIAL_TAG, data)) => match &self.credentials {
                Some(credentials) => credentials.decrypt(&self.name, data).await,
                None => Err(SecureEnclaveError::Other(
                    "systemd-creds is unavailable".to_owned(),
                )),
            },
            Some((&PLAINTEXT_TAG, data)) => Ok(data.to_owned()),
            _ => Err(SecureEnclaveError::Other(
                "unknown encrypted data format".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALIAS: &str = "net.nymtech.vpn.test-key";

    fn plaintext_key() -> LinuxSecureEnclaveKey {
        LinuxSecureEnclaveKey {
            name: ALIAS.to_owned(),
            credentials: None,
        }
    }

    #[tokio::test]
    async fn stores_plaintext_without_systemd_creds() {
        let key = plaintext_key();

        let encrypted = key.encrypt(b"secret").await.unwrap();

        assert_eq!(encrypted, [&[PLAINTEXT_TAG][..], b"secret"].concat());
        assert_eq!(key.decrypt(&encrypted).await.unwrap(), b"secret");
    }

    #[tokio::test]
    async fn rejects_credentials_without_systemd_creds() {
        let key = plaintext_key();

        assert!(key.decrypt(&[CREDENTIAL_TAG, b'a']).await.is_err());
        assert!(key.decrypt(&[]).await.is_err());
    }

    #[tokio::test]
    async fn rejects_user_presence() {
        let provider = LinuxSecureEnclaveProvider;

        assert!(provider.create_key(ALIAS, true).await.is_err());
    }

    #[tokio::test]
    #[ignore = "requires root and the systemd-creds varlink service"]
    async fn round_trips_with_systemd_creds() {
        let key = LinuxSecureEnclaveProvider
            .create_key(ALIAS, false)
            .await
            .unwrap();
        let data = vec![0x5a; 1024];

        let encrypted = key.encrypt(&data).await.unwrap();
        let decrypted = key.decrypt(&encrypted).await.unwrap();

        assert_eq!(encrypted[0], CREDENTIAL_TAG);
        assert_eq!(decrypted, data);
    }

    #[tokio::test]
    #[ignore = "requires root and the systemd-creds varlink service"]
    async fn authenticates_name_with_systemd_creds() {
        let key = LinuxSecureEnclaveProvider.get_key(ALIAS).await.unwrap();
        let other = LinuxSecureEnclaveProvider
            .get_key("net.nymtech.vpn.other-key")
            .await
            .unwrap();

        let encrypted = key.encrypt(b"secret").await.unwrap();

        assert!(other.decrypt(&encrypted).await.is_err());
    }

    #[tokio::test]
    #[ignore = "requires root and the systemd-creds varlink service"]
    async fn decrypts_plaintext_with_systemd_creds() {
        let key = LinuxSecureEnclaveProvider.get_key(ALIAS).await.unwrap();

        let decrypted = key
            .decrypt(&[&[PLAINTEXT_TAG][..], b"secret"].concat())
            .await;

        assert_eq!(decrypted.unwrap(), b"secret");
    }
}
