use std::{fs::File, path::Path};

pub const COSMOS_DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";

pub fn store_mnemonic<P: AsRef<Path> + Clone>(storage_path: P, mnemonic_phrase: &str) {
    let mnemonic = bip39::Mnemonic::parse(mnemonic_phrase).unwrap();
    let seed = mnemonic.to_seed("");
    // let prefix = "".to_string();
    let hd_path: cosmrs::bip32::DerivationPath = COSMOS_DERIVATION_PATH.parse().unwrap();
    let signing_key =
        cosmrs::crypto::secp256k1::SigningKey::derive_from_path(seed, &hd_path).unwrap();

    use std::io::Write;
    // let signing_key_hex = hex::encode(signing_key.to_bytes());

    let mut file = File::create(storage_path).unwrap();

    writeln!(file, "Mnemonic: {}", mnemonic_phrase).unwrap();
    writeln!(file, "Derivation Path: {}", COSMOS_DERIVATION_PATH).unwrap();
    // writeln!(file, "Signing Key (Hex): {}", signing_key_hex).unwrap();

    let prefix = "n";
    let wallet = nym_validator_client::DirectSecp256k1HdWallet::builder(prefix).build(mnemonic);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn test_store_mnemonic() {
        // dummy mnemonic
        let mnemonic = bip39::Mnemonic::generate_in(bip39::Language::English, 12).unwrap();
        println!("Mnemonic: {}", mnemonic);

        let path: PathBuf = "/tmp/test.txt".parse().unwrap();
        store_mnemonic(path, &mnemonic.to_string());
    }
}
