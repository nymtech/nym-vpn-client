// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub mod ephemeral;
pub mod on_disk;

pub use bip39::Mnemonic;

pub trait MnemonicStorageError: Error + Send + Sync + 'static {
    fn is_mnemonic_stored(&self) -> bool;
}

pub trait MnemonicStorage {
    type StorageError: MnemonicStorageError;

    #[allow(async_fn_in_trait)]
    async fn load_mnemonic(&self) -> Result<Mnemonic, Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn store_mnemonic(&self, mnemonic: Mnemonic) -> Result<(), Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn remove_mnemonic(&self) -> Result<(), Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn is_mnemonic_stored(&self) -> Result<bool, Self::StorageError> {
        self.load_mnemonic().await.map(|_| true).or(Ok(false))
    }
}

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct StoredMnemonic {
    // Identifier of the mnemonic.
    name: String,

    // The mnemonic itself.
    mnemonic: Mnemonic,

    // Nonce used to confirm the mnemonic
    nonce: Nonce,
}

type Nonce = u32;
