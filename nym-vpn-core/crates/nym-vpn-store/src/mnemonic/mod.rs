// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use nym_vpn_lib_types::Mnemonic;
use serde::{Deserialize, Serialize};

pub mod ephemeral;
pub mod on_disk;

#[async_trait::async_trait]
pub trait MnemonicStorage {
    type StorageError: Error + Send + Sync + 'static;

    /// Returns `None` when no mnemonic is stored.
    async fn load_mnemonic(&self) -> Result<Option<StoredMnemonic>, Self::StorageError>;

    /// Fails if a mnemonic is already stored.
    async fn store_mnemonic(&self, mnemonic: StoredMnemonic) -> Result<(), Self::StorageError>;

    async fn remove_mnemonic(&self) -> Result<(), Self::StorageError>;

    async fn is_mnemonic_stored(&self) -> Result<bool, Self::StorageError>;
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredMnemonic {
    pub mnemonic: Mnemonic,
}

impl std::fmt::Debug for StoredMnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredMnemonic")
            .field("mnemonic", &"[redacted]")
            .finish()
    }
}

#[cfg(test)]
pub(crate) mod test_fixtures {
    use super::*;

    pub(crate) fn mnemonic_fixture() -> Mnemonic {
        "kiwi ketchup mix canvas curve ribbon congress method feel frozen act annual aunt comfort side joy mesh palace tennis cannon orange name tortoise piece".parse().unwrap()
    }

    pub(crate) fn stored_mnemonic_fixture() -> StoredMnemonic {
        StoredMnemonic {
            mnemonic: mnemonic_fixture(),
        }
    }
}
