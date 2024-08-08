// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::Mutex;

use super::{MnemonicStorage, StoredMnemonic};

struct InMemoryMnemonicStorage {
    mnemonic: Mutex<Option<StoredMnemonic>>,
}

#[derive(Debug, thiserror::Error)]
enum InMemoryMnemonicStorageError {
    #[error("No mnemonic stored")]
    NoMnemonicStored,
}

impl MnemonicStorage for InMemoryMnemonicStorage {
    type StorageError = InMemoryMnemonicStorageError;

    async fn store_mnemonic(
        &self,
        mnemonic: bip39::Mnemonic,
    ) -> Result<(), InMemoryMnemonicStorageError> {
        let stored_mnemonic = StoredMnemonic { mnemonic };
        self.mnemonic.lock().await.replace(stored_mnemonic);
        Ok(())
    }

    async fn load_mnemonic(&self) -> Result<bip39::Mnemonic, InMemoryMnemonicStorageError> {
        Ok(self
            .mnemonic
            .lock()
            .await
            .as_ref()
            .unwrap()
            .mnemonic
            .clone())
    }
}
