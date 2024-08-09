// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::error::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub mod ephemeral;
pub mod on_disk;

pub use bip39::Mnemonic;

pub trait MnemonicStorage {
    type StorageError: Error;

    #[allow(async_fn_in_trait)]
    async fn load_mnemonic(&self) -> Result<Mnemonic, Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn store_mnemonic(&self, mnemonic: Mnemonic) -> Result<(), Self::StorageError>;
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
