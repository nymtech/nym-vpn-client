// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs::File, path::PathBuf};

use super::{MnemonicStorage, StoredMnemonic};

#[derive(Debug, thiserror::Error)]
pub enum OnDiskMnemonicStorageError {
    #[error("No mnemonic stored")]
    NoMnemonicStored,
    // #[error("Failed to read mnemonic from file")]
    // ReadError(#[from] std::io::Error),
    // #[error("Failed to write mnemonic to file")]
    // WriteError(#[from] std::io::Error),
}

pub struct OnDiskMnemonicStorage {
    path: PathBuf,
}

impl OnDiskMnemonicStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl MnemonicStorage for OnDiskMnemonicStorage {
    type StorageError = OnDiskMnemonicStorageError;

    async fn store_mnemonic(
        &self,
        mnemonic: bip39::Mnemonic,
    ) -> Result<(), OnDiskMnemonicStorageError> {
        let stored_mnemonic = StoredMnemonic { mnemonic };

        let file = File::create(&self.path).unwrap();
        serde_json::to_writer(file, &stored_mnemonic).unwrap();
        Ok(())
    }

    async fn load_mnemonic(&self) -> Result<bip39::Mnemonic, OnDiskMnemonicStorageError> {
        let file = File::open(&self.path).unwrap();
        let stored_mnemonic: StoredMnemonic = serde_json::from_reader(file).unwrap();
        Ok(stored_mnemonic.mnemonic.clone())
    }
}
