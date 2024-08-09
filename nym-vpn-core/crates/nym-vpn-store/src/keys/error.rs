// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("failed to load device keys")]
    Load {
        path: PathBuf,
        error: Box<dyn std::error::Error>,
    },

    #[error("failed to create device keys")]
    Create {
        path: PathBuf,
        error: Box<dyn std::error::Error>,
    },

    #[error("failed to store device keys")]
    Store {
        path: PathBuf,
        error: Box<dyn std::error::Error>,
    },

    // WIP: feels malplaced
    #[error("invalid key pair, one is missing: {path}")]
    InvalidKeyPair { path: PathBuf },
}
