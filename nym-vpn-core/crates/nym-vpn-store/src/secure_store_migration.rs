// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    error::Error,
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
};

use nym_vpn_api_client::types::{VpnAccount, VpnAccountMode};
use nym_vpn_lib_types::{Mnemonic, StoredAccountMode, VpnAccountSummary};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::SyncIoBridge;

use crate::{
    device_state::{
        DeviceAccountError, DeviceStateStorage, StoredAccountIdentity, StoredDeviceState,
    },
    mnemonic::{MnemonicStorage, StoredMnemonic},
};

#[derive(Debug, thiserror::Error)]
pub enum SecureStoreMigrationError {
    #[error("failed to instantiate device account")]
    CreateDeviceAccount(#[from] DeviceAccountError),

    #[error("failed to load vpn account")]
    LoadVpnAccount(#[source] Box<dyn Error + Send + Sync>),

    #[error("failed to create ecash keypair")]
    CreateEcashKeyPair(#[source] Box<dyn Error + Send + Sync>),

    #[error("failed to parse legacy account")]
    ParseLegacyAccount(#[source] serde_json::Error),

    #[error("failed to {ctx}")]
    MnemonicStore {
        ctx: &'static str,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("failed to store device state")]
    StoreDeviceState(#[source] Box<dyn Error + Send + Sync>),

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

pub struct SecureStoreMigrationManager<'a, MnemonicStoreType, DeviceStateStoreType> {
    mnemonic_json_path: PathBuf,
    account_summary_json_path: PathBuf,
    mnemonic_storage: &'a MnemonicStoreType,
    device_state_storage: &'a DeviceStateStoreType,
}

impl<'a, MnemonicStoreType, DeviceStateStoreType>
    SecureStoreMigrationManager<'a, MnemonicStoreType, DeviceStateStoreType>
where
    MnemonicStoreType: MnemonicStorage,
    DeviceStateStoreType: DeviceStateStorage,
{
    pub fn new(
        mnemonic_json_path: impl Into<PathBuf>,
        account_summary_json_path: impl Into<PathBuf>,
        mnemonic_storage: &'a MnemonicStoreType,
        device_state_storage: &'a DeviceStateStoreType,
    ) -> Self {
        Self {
            mnemonic_json_path: mnemonic_json_path.into(),
            account_summary_json_path: account_summary_json_path.into(),
            mnemonic_storage,
            device_state_storage,
        }
    }

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

        self.remove_plain_files().await;
        Ok(())
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
        let account_summary: Option<VpnAccountSummary> = self.read_account_summary().await;

        let LegacyAccount { mnemonic, mode } =
            self.read_legacy_account(plain_mnemonic_file).await?;

        let account_identity = StoredAccountIdentity::new(mnemonic.clone(), mode)?;
        let vpn_account = VpnAccount::new(
            mnemonic.clone(),
            VpnAccountMode::from(account_identity.mode),
        )
        .map_err(|err| SecureStoreMigrationError::LoadVpnAccount(Box::new(err)))?;

        let ecash_keypair = vpn_account
            .create_ecash_keypair()
            .map_err(|err| SecureStoreMigrationError::CreateEcashKeyPair(Box::new(err)))?;

        let device_state = StoredDeviceState::LoggedIn {
            account_identity,
            ecash_keypair,
            summary: account_summary,
        };

        self.mnemonic_storage
            .store_mnemonic(StoredMnemonic { mnemonic })
            .await
            .map_err(|err| SecureStoreMigrationError::MnemonicStore {
                ctx: "store mnemonic",
                source: Box::new(err),
            })?;

        self.device_state_storage
            .store_device_state(device_state)
            .await
            .map_err(|err| SecureStoreMigrationError::StoreDeviceState(Box::new(err)))
    }

    /// Remove plaintext files (mnemonic, summary)
    async fn remove_plain_files(&self) {
        remove_file_if_exists(&self.mnemonic_json_path).await;
        remove_file_if_exists(&self.account_summary_json_path).await;
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

    async fn read_account_summary(&self) -> Option<VpnAccountSummary> {
        let f = tokio::fs::File::open(&self.account_summary_json_path)
            .await
            .map(Some)
            .or_else(|err| {
                if err.kind() == IoErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(err)
                }
            })
            .inspect_err(|err| {
                tracing::warn!(
                    "Failed to read account_summary at {}: {}",
                    self.account_summary_json_path.display(),
                    err
                )
            })
            .ok()??;

        let reader: SyncIoBridge<File> = SyncIoBridge::new(f);

        tokio::task::spawn_blocking(move || serde_json::from_reader(reader))
            .await
            .inspect_err(|err| tracing::warn!("Failed to join account summary task: {err}"))
            .ok()?
            .inspect_err(|err| tracing::warn!("Failed to read account summary: {err}"))
            .ok()
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

#[cfg(test)]
mod tests {

    use std::assert_matches;

    use super::*;
    use crate::{
        device_state::ephemeral::InMemoryDeviceStateStorage,
        mnemonic::{
            ephemeral::InMemoryMnemonicStorage, on_disk::OnDiskMnemonicStorage,
            test_fixtures::stored_mnemonic_fixture,
        },
        store_ciphers::noop_cipher::NoopStoreCipher,
    };

    #[tokio::test]
    async fn test_migration_without_summary() {
        let storage_dir = tempfile::TempDir::new().unwrap();
        let mnemonic_json_path = storage_dir.path().join("mnemonic.json");
        let account_summary_path = storage_dir.path().join("account-summary.json");

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
        let device_state_store = InMemoryDeviceStateStorage::default();

        let storage = SecureStoreMigrationManager::new(
            mnemonic_json_path,
            account_summary_path,
            &mnemonic_store,
            &device_state_store,
        );

        assert_matches!(storage.should_migrate().await, Ok(true));
        assert_matches!(storage.migrate().await, Ok(()));

        assert_matches!(mnemonic_store.load_mnemonic().await, Ok(Some(v)) if v == stored_mnemonic);
        assert_matches!(
            device_state_store.load_device_state().await,
            Ok(StoredDeviceState::LoggedIn {
                account_identity: StoredAccountIdentity {
                    id: todo!("tbd"),
                    pub_key: todo!("tbd"),
                    mode: StoredAccountMode::Decentralised,
                    signature_base64: todo!("tbd")
                },
                summary: None
            })
        )
    }

    #[tokio::test]
    async fn test_migration_with_summary() {
        let storage_dir = tempfile::TempDir::new().unwrap();
        let mnemonic_json_path = storage_dir.path().join("mnemonic.json");
        let account_summary_path = storage_dir.path().join("account-summary.json");

        let stored_mnemonic = stored_mnemonic_fixture();
        let json = serde_json::json!({
            "name": "default",
            "mnemonic": stored_mnemonic.mnemonic.to_string(),
            "mode": "api",
            "nonce": 0,
        });
        tokio::fs::write(&mnemonic_json_path, json.to_string())
            .await
            .unwrap();

        let json = serde_json::json!({
            "traffic_used_gb": 12,
            "traffic_limit_gb": 100,
            "traffic_reset_time": "2026-11-01T00:00:00Z",
            "fair_usage_data_unavailable": false,
            "account_addr": "n1qypqxpq9qcrsszg2pvxq6rs0zqg3yyc5lzv7xu",
            "canonical_account_addr": null,
            "auth_methods": [],
            "account_mode": "api",
            "subscription": null,
            "is_subscription_stacked": false,
            "account_status": "Active",
            "remaining_devices": 4,
            "is_device_active": true,
            "stale": false,
            "last_synced_utc": "2026-10-22T12:00:00Z",
        });
        tokio::fs::write(&account_summary_path, json.to_string())
            .await
            .unwrap();

        let mnemonic_store = InMemoryMnemonicStorage::default();
        let device_state_store = InMemoryDeviceStateStorage::default();

        let storage = SecureStoreMigrationManager::new(
            mnemonic_json_path,
            account_summary_path,
            &mnemonic_store,
            &device_state_store,
        );

        assert_matches!(storage.should_migrate().await, Ok(true));
        assert_matches!(storage.migrate().await, Ok(()));

        assert_matches!(mnemonic_store.load_mnemonic().await, Ok(Some(v)) if v == stored_mnemonic);
        assert_matches!(
            device_state_store.load_device_state().await,
            Ok(StoredDeviceState::LoggedIn {
                account_identity: StoredAccountIdentity { id: todo!(), pub_key: todo!(), mode: StoredAccountMode::Api, signature_base64: todo!() },
                summary: Some(summary),
            }) if summary.account_addr == "n1qypqxpq9qcrsszg2pvxq6rs0zqg3yyc5lzv7xu"
                && summary.traffic_used_gb == 12
                && summary.remaining_devices == 4
        )
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_skip_repeat_migration() {
        let storage_dir = tempfile::TempDir::new().unwrap();
        let mnemonic_json_path = storage_dir.path().join("mnemonic.json");
        let mnemonic_bin_path = storage_dir.path().join("mnemonic.bin");
        let account_summary_path = storage_dir.path().join("account-summary.json");

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
        let device_state_store = InMemoryDeviceStateStorage::default();

        assert_matches!(
            mnemonic_store.store_mnemonic(stored_mnemonic.clone()).await,
            Ok(())
        );

        let storage = SecureStoreMigrationManager::new(
            mnemonic_json_path,
            account_summary_path,
            &mnemonic_store,
            &device_state_store,
        );

        assert_matches!(storage.should_migrate().await, Ok(true));
        assert_matches!(storage.migrate().await, Ok(()));

        assert_matches!(mnemonic_store.load_mnemonic().await, Ok(Some(v)) if v == stored_mnemonic);
        assert_matches!(
            device_state_store.load_device_state().await,
            Ok(StoredDeviceState::LoggedOut)
        );

        assert_matches!(storage.should_migrate().await, Ok(false));
        assert_matches!(storage.migrate().await, Ok(()));
    }
}
