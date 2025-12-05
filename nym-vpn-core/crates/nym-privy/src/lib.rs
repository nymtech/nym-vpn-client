// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bip39::Mnemonic;
use sha2::{Digest, Sha256};

use crate::error::PrivyError;

pub mod error;

pub fn hex_signature_to_mnemonic(hex_signature: &str) -> Result<Mnemonic, PrivyError> {
    let bytes_signature = hex::decode(hex_signature)?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes_signature);
    let hashed_signature = hasher.finalize();

    let mnemonic = Mnemonic::from_entropy(&hashed_signature)?;

    Ok(mnemonic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_signature() {
        assert!(hex_signature_to_mnemonic("deadbeef").is_ok());
        assert!(hex_signature_to_mnemonic("invalidhex").is_err());
    }
}
