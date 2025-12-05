// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bip39::Mnemonic;
use nym_vpn_lib_types::LoginSecret;

use crate::error::UtilsError;

pub mod error;

pub fn parse_secret(secret: &LoginSecret) -> Result<Mnemonic, UtilsError> {
    let mnemonic = match secret {
        LoginSecret::Mnemonic(mnemonic) => Mnemonic::parse(mnemonic)?,
        LoginSecret::PrivyHexSignature(signature) => {
            nym_privy::hex_signature_to_mnemonic(signature)?
        }
    };

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
    }
}
