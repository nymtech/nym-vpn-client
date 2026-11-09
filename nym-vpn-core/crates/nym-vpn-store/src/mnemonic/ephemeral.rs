// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::Mutex;

use super::{MnemonicStorage, StoredMnemonic};

#[derive(Default)]
pub struct InMemoryMnemonicStorage {
    mnemonic: Mutex<Option<StoredMnemonic>>,
}

#[derive(Debug, thiserror::Error)]
pub enum InMemoryMnemonicStorageError {
    #[error("no mnemonic stored")]
    NoMnemonicStored,

    #[error("mnemonic already stored")]
    MnemonicAlreadyStored,
}

#[async_trait::async_trait]
impl MnemonicStorage for InMemoryMnemonicStorage {
    type StorageError = InMemoryMnemonicStorageError;

    async fn load_mnemonic(&self) -> Result<Option<StoredMnemonic>, InMemoryMnemonicStorageError> {
        Ok(self.mnemonic.lock().await.clone())
    }

    async fn store_mnemonic(
        &self,
        mnemonic: StoredMnemonic,
    ) -> Result<(), InMemoryMnemonicStorageError> {
        let mut handle: tokio::sync::MutexGuard<'_, Option<StoredMnemonic>> =
            self.mnemonic.lock().await;

        if handle.is_none() {
            *handle = Some(mnemonic);
            Ok(())
        } else {
            Err(InMemoryMnemonicStorageError::MnemonicAlreadyStored)
        }
    }

    async fn remove_mnemonic(&self) -> Result<(), InMemoryMnemonicStorageError> {
        self.mnemonic
            .lock()
            .await
            .take()
            .map(drop)
            .ok_or(InMemoryMnemonicStorageError::NoMnemonicStored)
    }

    async fn is_mnemonic_stored(&self) -> Result<bool, InMemoryMnemonicStorageError> {
        Ok(self.mnemonic.lock().await.is_some())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;
    use crate::mnemonic::test_fixtures::{mnemonic_fixture, stored_mnemonic_fixture};

    #[tokio::test]
    async fn store_and_load_mnemonic() {
        let storage = InMemoryMnemonicStorage::default();
        storage
            .store_mnemonic(stored_mnemonic_fixture())
            .await
            .unwrap();

        let loaded = storage.load_mnemonic().await.unwrap();
        assert_matches!(loaded, Some(stored) if stored.mnemonic == mnemonic_fixture());
    }

    #[tokio::test]
    async fn load_non_existing_mnemonic_returns_none() {
        let storage = InMemoryMnemonicStorage::default();

        let loaded = storage.load_mnemonic().await.unwrap();
        assert_matches!(loaded, None);
    }

    #[tokio::test]
    async fn store_twice_fails() {
        let storage = InMemoryMnemonicStorage::default();
        storage
            .store_mnemonic(stored_mnemonic_fixture())
            .await
            .unwrap();

        let result = storage.store_mnemonic(stored_mnemonic_fixture()).await;
        assert_matches!(
            result,
            Err(InMemoryMnemonicStorageError::MnemonicAlreadyStored)
        );
    }

    #[tokio::test]
    async fn remove_mnemonic() {
        let storage = InMemoryMnemonicStorage::default();
        storage
            .store_mnemonic(stored_mnemonic_fixture())
            .await
            .unwrap();
        storage.remove_mnemonic().await.unwrap();

        assert!(!storage.is_mnemonic_stored().await.unwrap());
        assert_matches!(
            storage.remove_mnemonic().await,
            Err(InMemoryMnemonicStorageError::NoMnemonicStored)
        );
    }
}
