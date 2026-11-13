// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod database_key;
pub mod device_state;
pub mod encrypted_file;
pub mod keys;
pub mod mnemonic;
pub mod secure_store_migration;
pub mod store_ciphers;

pub trait VpnStorage:
    mnemonic::MnemonicStorage + keys::device::DeviceKeyStore + device_state::DeviceStateStorage
{
}

#[cfg(test)]
mod tests {
    use sqlx::{ConnectOptions, sqlite::SqliteConnectOptions};

    /// Guards the CommonCrypto provider selected for Apple platforms in Cargo.toml. SQLCipher
    /// switches to OpenSSL there if `OPENSSL_DIR` (or `OPENSSL_LIB_DIR` + `OPENSSL_INCLUDE_DIR`,
    /// optionally target-prefixed) is set at build time, or if another crate enables
    /// `vendored-openssl`. Elsewhere both system and vendored OpenSSL report `openssl`.
    #[tokio::test]
    async fn sqlcipher_uses_expected_crypto_provider() {
        let expected = if cfg!(target_vendor = "apple") {
            "commoncrypto"
        } else {
            "openssl"
        };
        let tempdir = tempfile::tempdir().unwrap();
        let opts = SqliteConnectOptions::new()
            .filename(tempdir.path().join("provider.sqlite"))
            .create_if_missing(true);
        let key = nym_sqlcipher::DatabaseKey::new([0x42; nym_sqlcipher::KEY_LEN]);
        let mut conn = nym_sqlcipher::apply_key(opts, Some(&key))
            .await
            .unwrap()
            .connect()
            .await
            .unwrap();

        let provider: String = sqlx::query_scalar("PRAGMA cipher_provider")
            .fetch_one(&mut conn)
            .await
            .unwrap();

        assert_eq!(provider, expected);
    }
}
