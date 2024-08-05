use std::path::Path;

use bip39::Mnemonic;
// use cosmrs::crypto::secp256k1::SigningKey;

pub async fn store_mnemonic<P: AsRef<Path> + Clone>(
    path: P,
    mnemonic_phrase: &str,
) -> Result<(), String> {
    let mnemonic = Mnemonic::from_str(mnemonic_phrase).map_err(|e| e.to_string())?;
    let seed = bip39::Seed::new(&mnemonic, "");
    let signing_key = SigningKey::new_from_seed(seed.as_bytes());

    todo!();
}
