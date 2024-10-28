// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::Mutex;

use super::{MnemonicStorage, MnemonicStorageError, StoredMnemonic};

struct InMemoryMnemonicStorage {
    mnemonic: Mutex<Option<StoredMnemonic>>,
}

impl InMemoryMnemonicStorage {
    #[allow(unused)]
    fn new() -> Self {
        Self {
            mnemonic: Mutex::new(None),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum InMemoryMnemonicStorageError {
    #[error("no mnemonic stored")]
    NoMnemonicStored,

    #[error("mnemonic already stored")]
    MnemonicAlreadyStored,
}

impl MnemonicStorageError for InMemoryMnemonicStorageError {
    fn is_mnemonic_stored(&self) -> bool {
        match self {
            InMemoryMnemonicStorageError::NoMnemonicStored => false,
            InMemoryMnemonicStorageError::MnemonicAlreadyStored => true,
        }
    }
}

impl MnemonicStorage for InMemoryMnemonicStorage {
    type StorageError = InMemoryMnemonicStorageError;

    async fn store_mnemonic(
        &self,
        mnemonic: bip39::Mnemonic,
    ) -> Result<(), InMemoryMnemonicStorageError> {
        let name = "default".to_string();
        let nonce = 0;
        let stored_mnemonic = StoredMnemonic {
            name,
            mnemonic,
            nonce,
        };
        let mut handle = self.mnemonic.lock().await;

        // Store the mnemonic if it's currently None
        if handle.is_none() {
            *handle = Some(stored_mnemonic);
            Ok(())
        } else {
            Err(InMemoryMnemonicStorageError::MnemonicAlreadyStored)
        }
    }

    async fn load_mnemonic(&self) -> Result<bip39::Mnemonic, InMemoryMnemonicStorageError> {
        self.mnemonic
            .lock()
            .await
            .as_ref()
            .map(|stored| stored.mnemonic.clone())
            .ok_or(InMemoryMnemonicStorageError::NoMnemonicStored)
    }

    async fn remove_mnemonic(&self) -> Result<(), InMemoryMnemonicStorageError> {
        let mut handle = self.mnemonic.lock().await;

        if handle.is_some() {
            *handle = None;
            Ok(())
        } else {
            Err(InMemoryMnemonicStorageError::NoMnemonicStored)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn store_and_load_mnemonic() {
        let mnemonic = "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece";
        let mnemonic = bip39::Mnemonic::parse(mnemonic).unwrap();

        let storage = InMemoryMnemonicStorage::new();
        storage.store_mnemonic(mnemonic.clone()).await.unwrap();

        let loaded_mnemonic = storage.load_mnemonic().await.unwrap();
        assert_eq!(loaded_mnemonic, mnemonic);
    }
}
