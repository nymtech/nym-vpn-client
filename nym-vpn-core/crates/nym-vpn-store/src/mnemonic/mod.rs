// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub mod ephemeral;
pub mod on_disk;

pub use bip39::Mnemonic;

#[async_trait::async_trait]
pub trait MnemonicStorage {
    type StorageError: Error + Send + Sync + 'static;

    async fn load_mnemonic(&self) -> Result<Option<Mnemonic>, Self::StorageError>; // None means no error, but no mnemonic stored
    async fn store_mnemonic(&self, mnemonic: Mnemonic) -> Result<(), Self::StorageError>;
    async fn remove_mnemonic(&self) -> Result<(), Self::StorageError>;
    async fn is_mnemonic_stored(&self) -> Result<bool, Self::StorageError> {
        self.load_mnemonic()
            .await
            .map(|maybe_mnemonic| maybe_mnemonic.is_some())
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
