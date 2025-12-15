// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_crypto::hkdf;
use nym_validator_client::nyxd::bip32::secp256k1::sha2::Sha256;
use nym_vpn_store::account::Mnemonic;

use super::error::LoginError;

pub const PRIVY_DERIVATION_MESSAGE: &str = "DeriveAccount:NymVPN";
const HKDF_SALT: &str = "privy-bip44-derivation";
const HKDF_INFO: &str = "cosmos-entropy";

pub fn message_to_sign() -> String {
    hex::encode(PRIVY_DERIVATION_MESSAGE.as_bytes())
}

pub fn hex_signature_to_mnemonic(hex_signature: &str) -> Result<Mnemonic, LoginError> {
    let bytes_signature = hex::decode(hex_signature)?;

    let entropy = hkdf::extract_then_expand::<Sha256>(
        Some(HKDF_SALT.as_bytes()),
        &bytes_signature,
        Some(HKDF_INFO.as_bytes()),
        32,
    )
    .map_err(|_| LoginError::HkdfInvalidLength)?;

    let mnemonic = Mnemonic::from_entropy(&entropy)?;

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
