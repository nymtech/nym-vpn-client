// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};
use zeroize::Zeroize;

use super::ciphers::{StoreCipher, StoreCipherError};

/// Marker at the start of every encrypted item. Plaintext JSON, PEM, base58 and bincode never
/// begin with these bytes, so the loader can tell the two formats apart.
const MAGIC: &[u8; 9] = b"NYMENC.1\n";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("cipher error")]
    Cipher(#[from] StoreCipherError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct EncryptedFile<T: StoreCipher> {
    path: PathBuf,
    cipher: T,
}

impl<T> EncryptedFile<T>
where
    T: StoreCipher,
{
    pub fn new(path: PathBuf, cipher: T) -> Self {
        Self { path, cipher }
    }

    pub async fn read(&self) -> Result<Vec<u8>> {
        let mut data = {
            let mut file = OpenOptions::new().read(true).open(&self.path).await?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).await?;
            data
        };

        // Check if data has magic cookie, if it doesn't then it means the file is not encrypted
        // Write the data to the file to encrypt it
        if data.starts_with(MAGIC) {
            data.drain(0..MAGIC.len());
            Ok(self.cipher.decrypt(data).await?)
        } else {
            match self.write(data.clone()).await {
                Ok(_) => Ok(data),
                Err(err) => {
                    data.zeroize();
                    Err(Error::from(err))
                }
            }
        }
    }

    pub async fn write(&self, data: Vec<u8>) -> Result<()> {
        // Write file to temporary location first
        let temp_file = self.path.with_added_extension("tmp");
        {
            let cipherdata = self.cipher.encrypt(data).await?;
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&temp_file)
                .await?;

            file.write_all(MAGIC).await?;
            file.write_all(&cipherdata).await?;
            file.flush().await?;
        }
        // Rename temporary file to original location to support atomic writes
        tokio::fs::rename(&temp_file, &self.path).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAINTEXT: &[u8] = b"{\"mnemonic\":\"correct horse battery staple\"}";

    #[tokio::test]
    async fn write_then_read_returns_original_data() {
        let tempdir = tempfile::tempdir().unwrap();
        let file = encrypted_file(tempdir.path().join("store.json"));

        file.write(PLAINTEXT).await.unwrap();

        assert_eq!(file.read().await.unwrap(), PLAINTEXT);
    }

    #[tokio::test]
    async fn write_then_read_returns_empty_data() {
        let tempdir = tempfile::tempdir().unwrap();
        let file = encrypted_file(tempdir.path().join("store.json"));

        file.write(b"").await.unwrap();

        assert_eq!(file.read().await.unwrap(), b"");
    }

    #[tokio::test]
    async fn write_stores_magic_followed_by_ciphertext() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.json");
        let file = encrypted_file(path.clone());

        file.write(PLAINTEXT).await.unwrap();

        let contents = tokio::fs::read(&path).await.unwrap();
        let ciphertext = contents.strip_prefix(MAGIC).expect("missing magic");
        assert_eq!(ciphertext, XorCipher.encrypt(PLAINTEXT).await.unwrap());
        assert_ne!(ciphertext, PLAINTEXT);
    }

    #[tokio::test]
    async fn write_overwrites_existing_content() {
        let tempdir = tempfile::tempdir().unwrap();
        let file = encrypted_file(tempdir.path().join("store.json"));
        file.write(b"a much longer piece of data that is replaced")
            .await
            .unwrap();

        file.write(PLAINTEXT).await.unwrap();

        assert_eq!(file.read().await.unwrap(), PLAINTEXT);
    }

    #[tokio::test]
    async fn write_does_not_leave_temporary_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.json");
        let file = encrypted_file(path.clone());

        file.write(PLAINTEXT).await.unwrap();

        assert_eq!(file_names(tempdir.path()).await, ["store.json"]);
    }

    #[tokio::test]
    async fn write_failure_keeps_existing_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.json");
        encrypted_file(path.clone()).write(PLAINTEXT).await.unwrap();
        let contents = tokio::fs::read(&path).await.unwrap();

        let result = EncryptedFile::new(path.clone(), FailingCipher)
            .write(b"new data")
            .await;

        assert!(matches!(
            result,
            Err(Error::Cipher(StoreCipherError::AuthenticationFailed))
        ));
        assert_eq!(tokio::fs::read(&path).await.unwrap(), contents);
        assert_eq!(file_names(tempdir.path()).await, ["store.json"]);
    }

    #[tokio::test]
    async fn read_missing_file_fails() {
        let tempdir = tempfile::tempdir().unwrap();
        let file = encrypted_file(tempdir.path().join("store.json"));

        let result = file.read().await;

        assert!(
            matches!(result, Err(Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound)
        );
    }

    #[tokio::test]
    async fn read_plaintext_file_returns_data_and_encrypts_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.json");
        tokio::fs::write(&path, PLAINTEXT).await.unwrap();
        let file = encrypted_file(path.clone());

        assert_eq!(file.read().await.unwrap(), PLAINTEXT);

        let contents = tokio::fs::read(&path).await.unwrap();
        assert!(contents.starts_with(MAGIC));
        assert_ne!(contents, PLAINTEXT);
        assert_eq!(file.read().await.unwrap(), PLAINTEXT);
    }

    #[tokio::test]
    async fn read_plaintext_file_fails_when_encryption_fails() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.json");
        tokio::fs::write(&path, PLAINTEXT).await.unwrap();

        let result = EncryptedFile::new(path.clone(), FailingCipher).read().await;

        assert!(matches!(
            result,
            Err(Error::Cipher(StoreCipherError::AuthenticationFailed))
        ));
        assert_eq!(tokio::fs::read(&path).await.unwrap(), PLAINTEXT);
    }

    #[tokio::test]
    async fn read_fails_when_decryption_fails() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.json");
        encrypted_file(path.clone()).write(PLAINTEXT).await.unwrap();

        let result = EncryptedFile::new(path, FailingCipher).read().await;

        assert!(matches!(
            result,
            Err(Error::Cipher(StoreCipherError::AuthenticationFailed))
        ));
    }

    struct XorCipher;

    #[async_trait::async_trait]
    impl StoreCipher for XorCipher {
        async fn encrypt(&self, data: &[u8]) -> crate::ciphers::Result<Vec<u8>> {
            Ok(data.iter().map(|byte| byte ^ 0x5a).collect())
        }

        async fn decrypt(&self, data: &[u8]) -> crate::ciphers::Result<Vec<u8>> {
            self.encrypt(data).await
        }
    }

    struct FailingCipher;

    #[async_trait::async_trait]
    impl StoreCipher for FailingCipher {
        async fn encrypt(&self, _data: &[u8]) -> crate::ciphers::Result<Vec<u8>> {
            Err(StoreCipherError::AuthenticationFailed)
        }

        async fn decrypt(&self, _data: &[u8]) -> crate::ciphers::Result<Vec<u8>> {
            Err(StoreCipherError::AuthenticationFailed)
        }
    }

    fn encrypted_file(path: PathBuf) -> EncryptedFile<XorCipher> {
        EncryptedFile::new(path, XorCipher)
    }

    async fn file_names(dir: &std::path::Path) -> Vec<String> {
        let mut entries = tokio::fs::read_dir(dir).await.unwrap();
        let mut names = Vec::new();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
        names.sort();
        names
    }
}
