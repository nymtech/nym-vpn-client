// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_crypto::asymmetric::ed25519;
use rand::SeedableRng;
use zeroize::Zeroizing;

use nym_secure_enclave::{KeychainError, KeychainItem};

use crate::keys::device::{DeviceKeyStore, DeviceKeys, DeviceKeysPaths};

#[derive(Debug, thiserror::Error)]
pub enum KeychainKeysError {
    #[error("device key keychain error")]
    Keychain(#[from] KeychainError),

    #[error("invalid device key")]
    InvalidKey(#[source] ed25519::Ed25519RecoveryError),

    #[error("failed to load plaintext device keys")]
    LoadPlaintextKeys(#[source] std::io::Error),
}

/// Stores the device private key in the keychain.
pub struct KeychainKeys {
    item: KeychainItem,
    plaintext_paths: Option<DeviceKeysPaths>,
}

impl KeychainKeys {
    pub fn new(item: KeychainItem) -> Self {
        KeychainKeys {
            item,
            plaintext_paths: None,
        }
    }

    /// Moves plaintext PEM keys found at `paths` into the keychain when no keys are stored there.
    pub fn with_plaintext_migration(mut self, paths: DeviceKeysPaths) -> Self {
        self.plaintext_paths = Some(paths);
        self
    }

    async fn load_keychain_keys(&self) -> Result<Option<DeviceKeys>, KeychainKeysError> {
        let private_key = self.item.get_value().await?;
        private_key
            .map(|bytes| {
                let private_key = ed25519::PrivateKey::from_bytes(&bytes)
                    .map_err(KeychainKeysError::InvalidKey)?;
                Ok(DeviceKeys::from_keys(ed25519::KeyPair::from(private_key)))
            })
            .transpose()
    }

    async fn migrate_plaintext_keys(&self) -> Result<Option<DeviceKeys>, KeychainKeysError> {
        let Some(paths) = &self.plaintext_paths else {
            return Ok(None);
        };

        match nym_pemstore::load_keypair::<ed25519::KeyPair>(&paths.device_key_pair_path()) {
            Ok(keypair) => {
                tracing::info!("Moving device keys to the keychain");
                let keys = DeviceKeys::from_keys(keypair);
                self.store_keys(&keys).await?;
                self.remove_plaintext_keys();
                Ok(Some(keys))
            }
            // Another process may have migrated the keys since the keychain was checked.
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                self.load_keychain_keys().await
            }
            Err(err) => Err(KeychainKeysError::LoadPlaintextKeys(err)),
        }
    }

    fn remove_plaintext_keys(&self) {
        let paths = self.plaintext_paths.iter();
        for path in paths.flat_map(|paths| [paths.private_device_key(), paths.public_device_key()])
        {
            match std::fs::remove_file(path) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => tracing::warn!("Failed to remove {}: {err}", path.display()),
            }
        }
    }
}

#[async_trait::async_trait]
impl DeviceKeyStore for KeychainKeys {
    type StorageError = KeychainKeysError;

    async fn load_keys(&self) -> Result<Option<DeviceKeys>, KeychainKeysError> {
        match self.load_keychain_keys().await? {
            Some(keys) => Ok(Some(keys)),
            None => self.migrate_plaintext_keys().await,
        }
    }

    async fn store_keys(&self, keys: &DeviceKeys) -> Result<(), KeychainKeysError> {
        let private_key = Zeroizing::new(keys.device_keypair().private_key().to_bytes());
        Ok(self.item.set_value(&*private_key).await?)
    }

    async fn init_keys(&self, seed: Option<[u8; 32]>) -> Result<(), KeychainKeysError> {
        if self.load_keys().await?.is_some() {
            Ok(())
        } else {
            self.reset_keys(seed).await
        }
    }

    async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), KeychainKeysError> {
        let device_keys = if let Some(seed) = seed {
            let mut rng = rand::rngs::ChaCha20Rng::from_seed(seed);
            DeviceKeys::generate_new(&mut rng)
        } else {
            DeviceKeys::generate_new(&mut rand::rng())
        };
        self.store_keys(&device_keys).await
    }

    async fn remove_keys(&self) -> Result<(), KeychainKeysError> {
        self.remove_plaintext_keys();
        Ok(self.item.delete()?)
    }
}

#[cfg(test)]
mod tests {
    use nym_secure_enclave::{Keychain, KeychainAccess, LocalAuthentication};

    use super::*;

    const TEST_SERVICE: &str = "net.nymtech.vpn.tests.keychain-keys";

    fn keychain_item(account: &str) -> KeychainItem {
        Keychain::new(LocalAuthentication::new(), None).item(
            TEST_SERVICE,
            account,
            KeychainAccess::AfterFirstUnlock,
        )
    }

    fn public_key(keys: &DeviceKeys) -> String {
        keys.device_keypair().public_key().to_base58_string()
    }

    // The data protection keychain requires a signed binary with keychain entitlements.
    #[tokio::test]
    #[ignore = "requires keychain entitlements"]
    async fn stores_loads_and_removes_keys() {
        let store = KeychainKeys::new(keychain_item("test"));
        store.remove_keys().await.unwrap();
        assert!(store.load_keys().await.unwrap().is_none());

        let keys = DeviceKeys::generate_new(&mut rand::rng());
        store.store_keys(&keys).await.unwrap();
        let loaded = store.load_keys().await.unwrap().unwrap();
        assert_eq!(public_key(&loaded), public_key(&keys));

        let replacement = DeviceKeys::generate_new(&mut rand::rng());
        store.store_keys(&replacement).await.unwrap();
        let loaded = store.load_keys().await.unwrap().unwrap();
        assert_eq!(public_key(&loaded), public_key(&replacement));

        store.remove_keys().await.unwrap();
        assert!(store.load_keys().await.unwrap().is_none());
    }

    #[tokio::test]
    #[ignore = "requires keychain entitlements"]
    async fn moves_plaintext_keys_to_keychain() {
        let tempdir = tempfile::tempdir().unwrap();
        let paths = DeviceKeysPaths::new(tempdir.path());
        let keys = DeviceKeys::generate_new(&mut rand::rng());
        let store = KeychainKeys::new(keychain_item("migration-test"))
            .with_plaintext_migration(DeviceKeysPaths::new(tempdir.path()));
        store.remove_keys().await.unwrap();
        nym_pemstore::store_keypair(
            keys.device_keypair().as_ref(),
            &paths.device_key_pair_path(),
        )
        .unwrap();

        let loaded = store.load_keys().await.unwrap().unwrap();

        assert_eq!(public_key(&loaded), public_key(&keys));
        assert!(!paths.private_device_key().exists());
        assert!(!paths.public_device_key().exists());
        assert!(store.item.get_value().await.unwrap().is_some());

        store.remove_keys().await.unwrap();
    }
}
