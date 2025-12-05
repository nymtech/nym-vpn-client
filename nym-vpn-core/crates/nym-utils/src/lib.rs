// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bip39::Mnemonic;
use nym_vpn_lib_types::LoginSecret;
use sha2::{Digest, Sha256};

use crate::error::UtilsError;

pub mod error;

pub fn parse_secret(secret: &LoginSecret) -> Result<Mnemonic, UtilsError> {
    let bytes_signature = match secret {
        LoginSecret::Mnemonic(mnemonic) => return Ok(Mnemonic::parse(mnemonic)?),
        LoginSecret::PrivyHexSignature(hex_signature) => hex::decode(hex_signature)?,
    };

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
    fn parse_mnemonic() {
        let mnemonic = Mnemonic::generate(24).unwrap();
        let parsed_mnemonic = parse_secret(&LoginSecret::Mnemonic(mnemonic.to_string())).unwrap();
        assert_eq!(mnemonic, parsed_mnemonic);

        assert!(parse_secret(&LoginSecret::Mnemonic(String::from("invalid mnemonic"))).is_err());
    }

    #[test]
    fn parse_hex_signature() {
        let hex_signature = String::from(
            "a564a87ccbed5cb5be4929201e555f5b5e26cb01d300d621520d724e57c582c33fa374caf21fd0c5e3118d70d14894845a32acfee47da7f347a0b9a57cba07931c",
        );

        assert!(parse_secret(&LoginSecret::PrivyHexSignature(hex_signature)).is_ok());
        assert!(parse_secret(&LoginSecret::PrivyHexSignature(String::from("invalidhex"))).is_err());
        assert!(parse_secret(&LoginSecret::PrivyHexSignature(String::from("deadbeef"))).is_ok());
    }
}
