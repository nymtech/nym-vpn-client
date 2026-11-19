// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::sync::Arc;
use std::{
    error::Error,
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
};

#[cfg(target_os = "android")]
use nym_secure_enclave::{SecureEnclave, SecureEnclaveError};
use nym_vpn_account::derive_ecash_keypair;
use nym_vpn_lib_types::{Mnemonic, StoredAccountMode};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::SyncIoBridge;
use typed_builder::TypedBuilder;

use crate::{
    account::{
        AccountInformationStorage, DeviceAccountError, StoredAccountIdentity,
        StoredAccountInformation,
    },
    mnemonic::{MnemonicStorage, StoredMnemonic},
};

#[derive(Debug, thiserror::Error)]
pub enum SecureStoreMigrationError {
    #[error("failed to instantiate device account")]
    CreateDeviceAccount(#[from] DeviceAccountError),

    #[error("failed to create ecash keypair")]
    CreateEcashKeyPair(#[source] nym_vpn_account::Error),

    #[error("failed to parse legacy account")]
    ParseLegacyAccount(#[source] serde_json::Error),

    #[error("failed to {ctx}")]
    MnemonicStore {
        ctx: &'static str,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("failed to store account information")]
    StoreAccountInformation(#[source] Box<dyn Error + Send + Sync>),

    #[cfg(target_os = "android")]
    #[error("failed to unlock keychain")]
    UnlockKeychain(#[source] SecureEnclaveError),

    #[error("failed to join blocking task")]
    Join(#[source] tokio::task::JoinError),

    #[error("failed to {ctx}: {}", path.display())]
    Io {
        ctx: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl SecureStoreMigrationError {
    fn is_parse_error(&self) -> bool {
        matches!(self, Self::ParseLegacyAccount(..))
    }
}

pub type Result<T, E = SecureStoreMigrationError> = std::result::Result<T, E>;

/// Account as stored in the plain mnemonic JSON file. Other fields of the legacy format are ignored.
#[derive(Deserialize)]
struct LegacyAccount {
    mnemonic: Mnemonic,
    #[serde(default)]
    mode: StoredAccountMode,
}

/// Moves the account from the plain mnemonic JSON file to the secure stores.
///
/// On Android, the keychain is unlocked before the mnemonic is encrypted, since its key requires
/// user presence for encryption.
#[derive(TypedBuilder)]
pub struct SecureStoreMigrationManager<'a, MnemonicStoreType, AccountStoreType> {
    #[builder(setter(into))]
    mnemonic_json_path: PathBuf,

    mnemonic_storage: &'a MnemonicStoreType,

    account_storage: &'a AccountStoreType,

    #[cfg(target_os = "android")]
    secure_enclave: Arc<SecureEnclave>,

    /// Shown to the user when authenticating to unlock the keychain.
    #[cfg(target_os = "android")]
    #[builder(setter(into))]
    unlock_reason: String,
}

impl<'a, MnemonicStoreType, AccountStoreType>
    SecureStoreMigrationManager<'a, MnemonicStoreType, AccountStoreType>
where
    MnemonicStoreType: MnemonicStorage,
    AccountStoreType: AccountInformationStorage,
{
    pub async fn should_migrate(&self) -> Result<bool> {
        tokio::fs::try_exists(&self.mnemonic_json_path)
            .await
            .map_err(|source| SecureStoreMigrationError::Io {
                ctx: "check file existence",
                path: self.mnemonic_json_path.to_path_buf(),
                source,
            })
    }

    pub async fn migrate(&self) -> Result<()> {
        let Some(f) = self.open_plain_mnemonic().await? else {
            return Ok(());
        };

        if self.is_encrypted_mnemonic_stored().await? {
            tracing::warn!(
                "Encrypted mnemonic is already stored. Skip migration and remove leftovers"
            );
        } else {
            self.migrate_mnemonic(f).await.or_else(|err| {
                if err.is_parse_error() {
                    tracing::warn!("Failed to parse legacy account: {err}");
                    Ok(())
                } else {
                    Err(err)
                }
            })?;
        }

        self.remove_plain_mnemonic().await;
        Ok(())
    }

    #[cfg(target_os = "android")]
    async fn unlock_keychain(&self) -> Result<()> {
        self.secure_enclave
            .unlock(&self.unlock_reason)
            .await
            .map_err(SecureStoreMigrationError::UnlockKeychain)
    }

    async fn open_plain_mnemonic(&self) -> Result<Option<File>> {
        tokio::fs::File::open(&self.mnemonic_json_path)
            .await
            .map(Some)
            .or_else(|err| {
                if err.kind() == IoErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(SecureStoreMigrationError::Io {
                        ctx: "open plain mnemonic",
                        path: self.mnemonic_json_path.clone(),
                        source: err,
                    })
                }
            })
    }

    /// Returns true if mnemonic is already stored in encrypted store
    async fn is_encrypted_mnemonic_stored(&self) -> Result<bool> {
        self.mnemonic_storage
            .is_mnemonic_stored()
            .await
            .map_err(|err| SecureStoreMigrationError::MnemonicStore {
                ctx: "check mnemonic stored",
                source: Box::new(err),
            })
    }

    /// Move mnemonic from plain file to encrypted store
    async fn migrate_mnemonic(&self, plain_mnemonic_file: File) -> Result<()> {
        let LegacyAccount { mnemonic, mode } =
            self.read_legacy_account(plain_mnemonic_file).await?;

        let account_identity = StoredAccountIdentity::new(mnemonic.clone(), mode)?;
        let ecash_keypair = derive_ecash_keypair(mnemonic.clone())
            .map_err(SecureStoreMigrationError::CreateEcashKeyPair)?;

        let account_information = StoredAccountInformation {
            account_identity,
            ecash_keypair,
        };

        // Android needs authorization for encryption
        // iOS: does not require authorization for encryption (only for decryption)
        #[cfg(target_os = "android")]
        self.unlock_keychain().await?;

        self.mnemonic_storage
            .store_mnemonic(StoredMnemonic { mnemonic })
            .await
            .map_err(|err| SecureStoreMigrationError::MnemonicStore {
                ctx: "store mnemonic",
                source: Box::new(err),
            })?;

        self.account_storage
            .store_account(account_information)
            .await
            .map_err(|err| SecureStoreMigrationError::StoreAccountInformation(Box::new(err)))
    }

    async fn remove_plain_mnemonic(&self) {
        remove_file_if_exists(&self.mnemonic_json_path).await;
    }

    /// Parse legacy account from plaintext mnemonic file
    async fn read_legacy_account(&self, file: File) -> Result<LegacyAccount> {
        let reader = SyncIoBridge::new(file);

        tokio::task::spawn_blocking(move || serde_json::from_reader(reader))
            .await
            .map_err(SecureStoreMigrationError::Join)?
            .map_err(|err| {
                if err.is_io() {
                    SecureStoreMigrationError::Io {
                        ctx: "read plain mnemonic",
                        path: self.mnemonic_json_path.clone(),
                        source: err.into(),
                    }
                } else {
                    SecureStoreMigrationError::ParseLegacyAccount(err)
                }
            })
    }
}

async fn remove_file_if_exists(path: &Path) {
    tokio::fs::remove_file(path)
        .await
        .inspect(|()| tracing::debug!("Removed {}", path.display()))
        .or_else(|err| {
            if err.kind() == IoErrorKind::NotFound {
                Ok(())
            } else {
                Err(err)
            }
        })
        .unwrap_or_else(|err| tracing::warn!("Failed to delete {}: {}", path.display(), err));
}

// Constructing the migration manager requires a keystore provider on Android.
#[cfg(all(test, not(target_os = "android")))]
mod tests {

    use std::assert_matches;

    use super::*;
    use crate::{
        account::ephemeral::InMemoryAccountStorage,
        mnemonic::{
            ephemeral::InMemoryMnemonicStorage, on_disk::OnDiskMnemonicStorage,
            test_fixtures::stored_mnemonic_fixture,
        },
        store_ciphers::noop_cipher::NoopStoreCipher,
    };
    use nym_credentials_interface::KeyPairUser;

    fn expected_account_keys(
        mnemonic: &Mnemonic,
        mode: StoredAccountMode,
    ) -> (StoredAccountIdentity, KeyPairUser) {
        let account_identity = StoredAccountIdentity::new(mnemonic.clone(), mode).unwrap();
        let ecash_keypair = derive_ecash_keypair(mnemonic.clone()).unwrap();
        (account_identity, ecash_keypair)
    }

    #[tokio::test]
    async fn test_migration() {
        let storage_dir = tempfile::TempDir::new().unwrap();
        let mnemonic_json_path = storage_dir.path().join("mnemonic.json");

        let stored_mnemonic = stored_mnemonic_fixture();
        let json = serde_json::json!({
            "name": "default",
            "mnemonic": stored_mnemonic.mnemonic.to_string(),
            "mode": "decentralised",
            "nonce": 0,
        });

        tokio::fs::write(&mnemonic_json_path, json.to_string())
            .await
            .unwrap();

        let mnemonic_store = InMemoryMnemonicStorage::default();
        let account_store = InMemoryAccountStorage::default();

        let storage = SecureStoreMigrationManager::builder()
            .mnemonic_json_path(mnemonic_json_path)
            .mnemonic_storage(&mnemonic_store)
            .account_storage(&account_store)
            .build();

        assert_matches!(storage.should_migrate().await, Ok(true));
        assert_matches!(storage.migrate().await, Ok(()));

        assert_matches!(mnemonic_store.load_mnemonic().await, Ok(Some(v)) if v == stored_mnemonic);

        let (account_identity, ecash_keypair) =
            expected_account_keys(&stored_mnemonic.mnemonic, StoredAccountMode::Decentralised);
        assert_eq!(
            account_store.load_account().await.unwrap(),
            Some(StoredAccountInformation {
                account_identity,
                ecash_keypair,
            })
        );
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_skip_repeat_migration() {
        let storage_dir = tempfile::TempDir::new().unwrap();
        let mnemonic_json_path = storage_dir.path().join("mnemonic.json");
        let mnemonic_bin_path = storage_dir.path().join("mnemonic.bin");

        let stored_mnemonic = stored_mnemonic_fixture();
        let json = serde_json::json!({
            "name": "default",
            "mnemonic": stored_mnemonic.mnemonic.to_string(),
            "mode": "decentralised",
            "nonce": 0,
        });

        tokio::fs::write(&mnemonic_json_path, json.to_string())
            .await
            .unwrap();

        let mnemonic_store = OnDiskMnemonicStorage::new(&mnemonic_bin_path, NoopStoreCipher);
        let account_store = InMemoryAccountStorage::default();

        assert_matches!(
            mnemonic_store.store_mnemonic(stored_mnemonic.clone()).await,
            Ok(())
        );

        let storage = SecureStoreMigrationManager::builder()
            .mnemonic_json_path(mnemonic_json_path)
            .mnemonic_storage(&mnemonic_store)
            .account_storage(&account_store)
            .build();

        assert_matches!(storage.should_migrate().await, Ok(true));
        assert_matches!(storage.migrate().await, Ok(()));

        assert_matches!(mnemonic_store.load_mnemonic().await, Ok(Some(v)) if v == stored_mnemonic);
        assert_matches!(account_store.load_account().await, Ok(None));

        assert_matches!(storage.should_migrate().await, Ok(false));
        assert_matches!(storage.migrate().await, Ok(()));
    }
}
