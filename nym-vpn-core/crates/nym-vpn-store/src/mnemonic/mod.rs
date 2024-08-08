// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::error::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

mod ephemeral;
mod on_disk;

pub trait MnemonicStorage {
    type StorageError: Error;

    #[allow(async_fn_in_trait)]
    async fn load_mnemonic(&self) -> Result<bip39::Mnemonic, Self::StorageError>;

    #[allow(async_fn_in_trait)]
    async fn store_mnemonic(&self, mnemonic: bip39::Mnemonic) -> Result<(), Self::StorageError>;
}

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct StoredMnemonic {
    mnemonic: bip39::Mnemonic,
}
