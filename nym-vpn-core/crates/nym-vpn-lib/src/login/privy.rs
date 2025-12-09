// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_crypto::Digest;
use nym_validator_client::nyxd::bip32::secp256k1::sha2::Sha256;
use nym_vpn_store::account::Mnemonic;

use super::error::LoginError;

pub const PRIVY_DERIVATION_MESSAGE: &str = "DeriveAccount:NymVPN";

pub fn message_to_sign() -> String {
    hex::encode(PRIVY_DERIVATION_MESSAGE.as_bytes())
}

pub fn hex_signature_to_mnemonic(hex_signature: &str) -> Result<Mnemonic, LoginError> {
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
