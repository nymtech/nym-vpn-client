// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
};

use nym_vpn_lib_types::StorableAccount;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use zeroize::Zeroizing;

use super::{AccountInformationStorage, DEFAULT_NAME, StoredAccount};
use crate::ciphers::{StoreCipher, StoreCipherError};

#[derive(Debug, thiserror::Error)]
pub enum OnDiskMnemonicStorageError {
    #[error("mnemonic already stored")]
    MnemonicAlreadyStored { path: PathBuf },

    #[error("failed to {ctx}: {path}")]
    Io {
        ctx: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serialize mnemonic")]
    Serialize(#[source] serde_json::Error),

    #[error("failed to deserialize mnemonic")]
    Deserialize(#[source] serde_json::Error),

    #[error("failed to encrypt data")]
    Encrypt(#[source] StoreCipherError),

    #[error("failed to encrypt data")]
    Decrypt(#[source] StoreCipherError),
}

pub type Result<T, E = OnDiskMnemonicStorageError> = std::result::Result<T, E>;

pub struct OnDiskAccountStorage {
    path: PathBuf,
    cipher: Box<dyn StoreCipher>,
}

impl OnDiskAccountStorage {
    pub fn new(path: impl AsRef<Path>, cipher: impl StoreCipher + 'static) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            cipher: Box::new(cipher),
        }
    }

    async fn check_mnemonic_exists(&self) -> Result<()> {
        tokio::fs::try_exists(&self.path)
            .await
            .map_err(|err| self.io_error("check if file exists", err))
            .and_then(|mnemonic_exists| {
                if mnemonic_exists {
                    Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored {
                        path: self.path.clone(),
                    })
                } else {
                    Ok(())
                }
            })
    }

    async fn create_parent_dirs(&self) -> Result<()> {
        tracing::trace!("Creating parent directories for: {}", self.path.display());

        let Some(parent) = self.path.parent() else {
            return Ok(());
        };

        tracing::trace!("Creating parent directory: {}", parent.display());
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| OnDiskMnemonicStorageError::Io {
                ctx: "create directory",
                path: parent.to_path_buf(),
                source: err,
            })?;

        #[cfg(unix)]
        {
            let permissions = Permissions::from_mode(0o700);
            tokio::fs::set_permissions(parent, permissions)
                .await
                .map_err(|err| OnDiskMnemonicStorageError::Io {
                    ctx: "set parent dir permissions",
                    path: parent.to_path_buf(),
                    source: err,
                })?;
        }

        Ok(())
    }

    async fn write_to_file(&self, ciphertext: &[u8]) -> Result<()> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)
            .await
            .map_err(|err| {
                if err.kind() == IoErrorKind::AlreadyExists {
                    OnDiskMnemonicStorageError::MnemonicAlreadyStored {
                        path: self.path.clone(),
                    }
                } else {
                    self.io_error("create file", err)
                }
            })?;

        #[cfg(unix)]
        file.set_permissions(Permissions::from_mode(0o600))
            .await
            .map_err(|err| self.io_error("set file permissions", err))?;

        file.write_all(&ciphertext)
            .await
            .map_err(|err| self.io_error("write file", err))
    }

    fn io_error(
        &self,
        operation: &'static str,
        source: std::io::Error,
    ) -> OnDiskMnemonicStorageError {
        OnDiskMnemonicStorageError::Io {
            ctx: operation,
            path: self.path.clone(),
            source,
        }
    }
}

#[async_trait::async_trait]
impl AccountInformationStorage for OnDiskAccountStorage {
    type StorageError = OnDiskMnemonicStorageError;

    async fn store_account(&self, account: StorableAccount) -> Result<()> {
        tracing::info!("Storing mnemonic to: {}", self.path.display());

        self.check_mnemonic_exists().await?;
        self.create_parent_dirs().await?;

        let ciphertext = {
            let nonce = 0;
            let stored_account = StoredAccount {
                name: DEFAULT_NAME.to_owned(),
                mnemonic: account.mnemonic,
                mode: account.mode,
                nonce,
            };

            let plaintext = serde_json::to_vec(&stored_account)
                .map(Zeroizing::new)
                .map_err(OnDiskMnemonicStorageError::Serialize)?;
            drop(stored_account);

            self.cipher
                .encrypt(&plaintext)
                .await
                .map_err(OnDiskMnemonicStorageError::Encrypt)?
        };

        self.write_to_file(&ciphertext).await
    }

    async fn load_account(&self) -> Result<Option<StorableAccount>> {
        tracing::debug!("Opening: {}", self.path.display());

        match File::open(&self.path).await {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)
                    .await
                    .map_err(|err| self.io_error("read file", err))?;

                let plaintext = self
                    .cipher
                    .decrypt(&buf)
                    .await
                    .map(Zeroizing::new)
                    .map_err(OnDiskMnemonicStorageError::Decrypt)?;

                serde_json::from_slice::<StoredAccount>(&plaintext)
                    .map_err(OnDiskMnemonicStorageError::Deserialize)
                    .map(StorableAccount::from)
                    .map(Some)
            }
            Err(err) => {
                if err.kind() == IoErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(self.io_error("open file", err))
                }
            }
        }
    }

    async fn remove_account(&self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }
        tokio::fs::remove_file(&self.path)
            .await
            .map_err(|err| self.io_error("remove file", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        account::{
            Nonce,
            test_fixtures::{account_fixture, mnemonic_fixture},
        },
        ciphers::noop_cipher::NoopStoreCipher,
    };
    use bip39::Mnemonic;
    use nym_vpn_lib_types::StoredAccountMode;
    use serde::{Deserialize, Serialize};

    #[tokio::test]
    async fn store_account() {
        let account = account_fixture();

        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");

        let account_storage = OnDiskAccountStorage::new(path.clone(), NoopStoreCipher::default());
        account_storage
            .store_account(account.clone())
            .await
            .unwrap();

        let stored_account = account_storage.load_account().await.unwrap();
        assert_eq!(Some(account), stored_account);
    }

    #[tokio::test]
    async fn store_twice_fails() {
        let account = account_fixture();

        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let account_storage = OnDiskAccountStorage::new(path.clone(), NoopStoreCipher::default());
        account_storage
            .store_account(account.clone())
            .await
            .unwrap();

        let result = account_storage.store_account(account).await;
        assert!(matches!(
            result,
            Err(OnDiskMnemonicStorageError::MnemonicAlreadyStored { .. })
        ));
    }

    #[tokio::test]
    async fn load_return_none_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let account_storage = OnDiskAccountStorage::new(path.clone(), NoopStoreCipher::default());
        let result = account_storage.load_account().await;
        assert!(matches!(result, Ok(None)));
    }

    #[tokio::test]
    async fn load_fails_if_bogus_empty_file_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.txt");
        let account_storage = OnDiskAccountStorage::new(path.clone(), NoopStoreCipher::default());
        let _ = File::create(&path).await.unwrap();
        let result = account_storage.load_account().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn load_of_legacy_mnemonics_still_works() -> anyhow::Result<()> {
        #[derive(Serialize, Deserialize)]
        struct LegacyStoredMnemonic {
            name: String,
            mnemonic: Mnemonic,
            nonce: Nonce,
        }

        let legacy = LegacyStoredMnemonic {
            name: "foomp".to_string(),
            mnemonic: mnemonic_fixture(),
            nonce: 0,
        };

        let tempdir = tempfile::tempdir()?;
        let path = tempdir.path().join("test.txt");

        // save legacy data
        tokio::fs::create_dir_all(tempdir.path()).await?;
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await?;
        file.write_all(&serde_json::to_vec(&legacy)?).await?;

        let expected = StorableAccount {
            mnemonic: mnemonic_fixture(),
            mode: StoredAccountMode::Api,
        };

        let account_storage = OnDiskAccountStorage::new(path.clone(), NoopStoreCipher::default());
        let loaded = account_storage.load_account().await?;
        assert_eq!(Some(expected), loaded);

        Ok(())
    }
}
