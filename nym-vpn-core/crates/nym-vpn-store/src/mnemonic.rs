use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use tokio::sync::Mutex;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct StoredMnemonic {
    mnemonic: bip39::Mnemonic,
}

trait MnemonicStorage {
    type StorageError: std::error::Error;

    async fn store_mnemonic(&self, mnemonic: bip39::Mnemonic) -> Result<(), Self::StorageError>;
    async fn load_mnemonic(&self) -> Result<bip39::Mnemonic, Self::StorageError>;
}

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

#[derive(Debug, thiserror::Error)]
enum OnDiskMnemonicStorageError {
    #[error("No mnemonic stored")]
    NoMnemonicStored,
    // #[error("Failed to read mnemonic from file")]
    // ReadError(#[from] std::io::Error),
    // #[error("Failed to write mnemonic to file")]
    // WriteError(#[from] std::io::Error),
}

struct OnDiskMnemonicStorage {
    path: PathBuf,
}

impl OnDiskMnemonicStorage {
    fn new(path: PathBuf) -> Self {
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

pub fn store_mnemonic<P: AsRef<Path> + Clone>(storage_path: P, mnemonic_phrase: &str) {
    let mnemonic = bip39::Mnemonic::parse(mnemonic_phrase).unwrap();

    // Check that we can create a keypair from it
    let prefix = "n";
    let _secp256k1_keypair =
        nym_validator_client::DirectSecp256k1HdWallet::from_mnemonic(prefix, mnemonic.clone());

    let stored_mnemonic = StoredMnemonic { mnemonic };
    let storage = OnDiskMnemonicStorage::new(storage_path.as_ref().to_path_buf());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn test_store_mnemonic() {
        // dummy mnemonic
        let mnemonic = bip39::Mnemonic::generate_in(bip39::Language::English, 12).unwrap();
        println!("Mnemonic: {}", mnemonic);

        let path: PathBuf = "/tmp/test.txt".parse().unwrap();
        store_mnemonic(path, &mnemonic.to_string());
    }
}
