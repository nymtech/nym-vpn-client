// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};
use zeroize::{Zeroize, ZeroizeOnDrop};

mod ephemeral;
mod on_disk;

pub trait MnemonicStorage {
    type StorageError: Error;

    async fn load_mnemonic(&self) -> Result<bip39::Mnemonic, Self::StorageError>;
    async fn store_mnemonic(&self, mnemonic: bip39::Mnemonic) -> Result<(), Self::StorageError>;
}

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct StoredMnemonic {
    mnemonic: bip39::Mnemonic,
}

pub fn store_mnemonic<P: AsRef<Path> + Clone>(storage_path: P, mnemonic_phrase: &str) {
    let mnemonic = bip39::Mnemonic::parse(mnemonic_phrase).unwrap();

    // Check that we can create a keypair from it
    let prefix = "n";
    let _secp256k1_keypair =
        nym_validator_client::DirectSecp256k1HdWallet::from_mnemonic(prefix, mnemonic.clone());

    let stored_mnemonic = StoredMnemonic { mnemonic };
    let storage = on_disk::OnDiskMnemonicStorage::new(storage_path.as_ref().to_path_buf());
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
