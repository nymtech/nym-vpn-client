// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use std::io::SeekFrom;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use zeroize::{Zeroize, Zeroizing};

use super::ciphers::{StoreCipher, StoreCipherError};

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
            Ok(self.cipher.decrypt(&data)?)
        } else {
            match self.write(&data).await {
                Ok(_) => Ok(data),
                Err(err) => {
                    data.zeroize();
                    Err(Error::from(err))
                }
            }
        }
    }

    /// Writes the encrypted data to the file, overwriting any existing content.
    pub async fn write(&self, data: &[u8]) -> Result<()> {
        // Write file to temporary location first
        let temp_file = self.path.with_added_extension("tmp");
        {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&temp_file)
                .await?;
            let cipherdata = self.cipher.encrypt(data)?;

            file.write_all(MAGIC).await?;
            file.write_all(&cipherdata).await?;
            file.flush().await?;
        }
        // Rename temporary file to original location to support atomic writes
        tokio::fs::rename(&temp_file, &self.path).await?;

        Ok(())
    }
}
