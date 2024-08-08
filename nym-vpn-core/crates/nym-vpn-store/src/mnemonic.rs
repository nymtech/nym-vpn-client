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
    async fn store_mnemonic(&self, mnemonic: bip39::Mnemonic);
    async fn read_mnemonic(&self) -> bip39::Mnemonic;
}

struct InMemoryMnemonicStorage {
    mnemonic: Mutex<Option<StoredMnemonic>>,
}

impl MnemonicStorage for InMemoryMnemonicStorage {
    async fn store_mnemonic(&self, mnemonic: bip39::Mnemonic) {
        let stored_mnemonic = StoredMnemonic { mnemonic };
        self.mnemonic.lock().await.replace(stored_mnemonic);
    }

    async fn read_mnemonic(&self) -> bip39::Mnemonic {
        self.mnemonic
            .lock()
            .await
            .as_ref()
            .unwrap()
            .mnemonic
            .clone()
    }
}

struct OnDiskMnemonicStorage {
    path: PathBuf,
}

impl OnDiskMnemonicStorage {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.try_into().unwrap(),
        }
    }
}

impl MnemonicStorage for OnDiskMnemonicStorage {
    async fn store_mnemonic(&self, mnemonic: bip39::Mnemonic) {
        let stored_mnemonic = StoredMnemonic { mnemonic };

        let file = File::create(&self.path).unwrap();
        serde_json::to_writer(file, &stored_mnemonic).unwrap();
    }

    async fn read_mnemonic(&self) -> bip39::Mnemonic {
        let file = File::open(&self.path).unwrap();
        let stored_mnemonic: StoredMnemonic = serde_json::from_reader(file).unwrap();
        stored_mnemonic.mnemonic
    }
}

pub fn store_mnemonic<P: AsRef<Path> + Clone>(storage_path: P, mnemonic_phrase: &str) {
    let mnemonic = bip39::Mnemonic::parse(mnemonic_phrase).unwrap();

    // Check that we can create a keypair from it
    let prefix = "n";
    let _secp256k1_keypair =
        nym_validator_client::DirectSecp256k1HdWallet::from_mnemonic(prefix, mnemonic.clone());

    let stored_mnemonic = StoredMnemonic { mnemonic };
    let storage = OnDiskMnemonicStorage::new(&storage_path);
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
