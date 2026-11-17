// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
};

use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use nym_vpn_lib_types::VpnAccountSummary;

use super::AccountSummaryStorage;
use crate::store_ciphers::{StoreCipher, StoreCipherError};

#[derive(Debug, thiserror::Error)]
pub enum OnDiskAccountSummaryStorageError {
    #[error("failed to {ctx}: {path}")]
    Io {
        ctx: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serialize account summary")]
    Serialize(#[source] serde_json::Error),

    #[error("failed to deserialize account summary")]
    Deserialize(#[source] serde_json::Error),

    #[error("failed to encrypt data")]
    Encrypt(#[source] StoreCipherError),

    #[error("failed to decrypt data")]
    Decrypt(#[source] StoreCipherError),
}

pub type Result<T, E = OnDiskAccountSummaryStorageError> = std::result::Result<T, E>;

pub struct OnDiskAccountSummaryStorage {
    path: PathBuf,
    cipher: Box<dyn StoreCipher>,
    legacy_plaintext_path: Option<PathBuf>,
}

impl OnDiskAccountSummaryStorage {
    pub fn new(path: impl AsRef<Path>, cipher: impl StoreCipher + 'static) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            cipher: Box::new(cipher),
            legacy_plaintext_path: None,
        }
    }

    /// Moves the plaintext JSON summary found at `path` into this storage when no summary is
    /// stored yet.
    pub fn with_legacy_plaintext(mut self, path: impl AsRef<Path>) -> Self {
        self.legacy_plaintext_path = Some(path.as_ref().to_path_buf());
        self
    }

    async fn migrate_legacy_summary(&self) -> Result<Option<VpnAccountSummary>> {
        let Some(path) = &self.legacy_plaintext_path else {
            return Ok(None);
        };

        let data = match tokio::fs::read(path).await {
            Ok(data) => data,
            Err(err) if err.kind() == IoErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(OnDiskAccountSummaryStorageError::Io {
                    ctx: "read legacy file",
                    path: path.clone(),
                    source,
                });
            }
        };

        let summary = serde_json::from_slice::<VpnAccountSummary>(&data)
            .inspect_err(|err| tracing::warn!("Failed to parse legacy account summary: {err}"))
            .ok();
        if let Some(summary) = &summary {
            tracing::info!("Moving account summary to encrypted storage");
            self.store_summary(summary).await?;
        }
        self.remove_legacy_summary().await?;

        Ok(summary)
    }

    async fn remove_legacy_summary(&self) -> Result<()> {
        let Some(path) = &self.legacy_plaintext_path else {
            return Ok(());
        };
        remove_file_if_exists(path).await
    }

    async fn create_parent_dirs(&self) -> Result<()> {
        let Some(parent) = self.path.parent() else {
            return Ok(());
        };

        tokio::fs::create_dir_all(parent).await.map_err(|err| {
            OnDiskAccountSummaryStorageError::Io {
                ctx: "create directory",
                path: parent.to_path_buf(),
                source: err,
            }
        })?;

        #[cfg(unix)]
        tokio::fs::set_permissions(parent, Permissions::from_mode(0o700))
            .await
            .map_err(|err| OnDiskAccountSummaryStorageError::Io {
                ctx: "set parent dir permissions",
                path: parent.to_path_buf(),
                source: err,
            })?;

        Ok(())
    }

    async fn write_atomic(&self, ciphertext: &[u8]) -> Result<()> {
        let temp_path = self.path.with_added_extension("tmp");
        let temp_io_error = |ctx, source| OnDiskAccountSummaryStorageError::Io {
            ctx,
            path: temp_path.clone(),
            source,
        };

        {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temp_path)
                .await
                .map_err(|err| temp_io_error("create file", err))?;

            #[cfg(unix)]
            file.set_permissions(Permissions::from_mode(0o600))
                .await
                .map_err(|err| temp_io_error("set file permissions", err))?;

            file.write_all(ciphertext)
                .await
                .map_err(|err| temp_io_error("write file", err))?;
            file.sync_all()
                .await
                .map_err(|err| temp_io_error("sync file", err))?;
        }

        tokio::fs::rename(&temp_path, &self.path)
            .await
            .map_err(|err| temp_io_error("rename file", err))
    }

    fn io_error(
        &self,
        ctx: &'static str,
        source: std::io::Error,
    ) -> OnDiskAccountSummaryStorageError {
        OnDiskAccountSummaryStorageError::Io {
            ctx,
            path: self.path.clone(),
            source,
        }
    }
}

#[async_trait::async_trait]
impl AccountSummaryStorage for OnDiskAccountSummaryStorage {
    type StorageError = OnDiskAccountSummaryStorageError;

    async fn load_summary(&self) -> Result<Option<VpnAccountSummary>> {
        match tokio::fs::read(&self.path).await {
            Ok(ciphertext) => {
                let plaintext = self
                    .cipher
                    .decrypt(&ciphertext)
                    .await
                    .map_err(OnDiskAccountSummaryStorageError::Decrypt)?;

                serde_json::from_slice(&plaintext)
                    .map(Some)
                    .map_err(OnDiskAccountSummaryStorageError::Deserialize)
            }
            Err(err) if err.kind() == IoErrorKind::NotFound => self.migrate_legacy_summary().await,
            Err(err) => Err(self.io_error("read file", err)),
        }
    }

    async fn store_summary(&self, summary: &VpnAccountSummary) -> Result<()> {
        tracing::debug!("Storing account summary to: {}", self.path.display());

        self.create_parent_dirs().await?;

        let plaintext =
            serde_json::to_vec(summary).map_err(OnDiskAccountSummaryStorageError::Serialize)?;
        let ciphertext = self
            .cipher
            .encrypt(&plaintext)
            .await
            .map_err(OnDiskAccountSummaryStorageError::Encrypt)?;

        self.write_atomic(&ciphertext).await
    }

    async fn remove_summary(&self) -> Result<()> {
        self.remove_legacy_summary().await?;
        remove_file_if_exists(&self.path).await
    }
}

async fn remove_file_if_exists(path: &Path) -> Result<()> {
    match tokio::fs::remove_file(path).await {
        Err(err) if err.kind() != IoErrorKind::NotFound => {
            Err(OnDiskAccountSummaryStorageError::Io {
                ctx: "remove file",
                path: path.to_path_buf(),
                source: err,
            })
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use nym_vpn_lib_types::VpnAccountStatus;
    use time::OffsetDateTime;

    use super::*;
    use crate::store_ciphers::noop_cipher::NoopStoreCipher;

    fn summary() -> VpnAccountSummary {
        VpnAccountSummary {
            traffic_used_gb: 1,
            traffic_limit_gb: 2,
            traffic_reset_time: None,
            fair_usage_data_unavailable: false,
            account_addr: "n1account".to_owned(),
            canonical_account_addr: None,
            auth_methods: Vec::new(),
            account_mode: None,
            subscription: None,
            is_subscription_stacked: false,
            account_status: VpnAccountStatus::Active,
            remaining_devices: 3,
            is_device_active: true,
            stale: false,
            last_synced_utc: OffsetDateTime::UNIX_EPOCH,
        }
    }

    fn storage(path: &Path) -> OnDiskAccountSummaryStorage {
        OnDiskAccountSummaryStorage::new(path, NoopStoreCipher)
    }

    const LEGACY_SUMMARY: &str = r#"{
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
        "last_synced_utc": "2026-10-22T12:00:00Z"
    }"#;

    #[tokio::test]
    async fn moves_legacy_summary() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("account_summary.bin");
        let legacy_path = tempdir.path().join("account_summary.json");
        tokio::fs::write(&legacy_path, LEGACY_SUMMARY)
            .await
            .unwrap();
        let storage = storage(&path).with_legacy_plaintext(&legacy_path);

        let summary = storage.load_summary().await.unwrap().unwrap();

        assert_eq!(
            summary.account_addr,
            "n1qypqxpq9qcrsszg2pvxq6rs0zqg3yyc5lzv7xu"
        );
        assert_eq!(summary.traffic_used_gb, 12);
        assert_eq!(summary.remaining_devices, 4);
        assert!(!legacy_path.exists());
        assert!(path.exists());
        assert_eq!(storage.load_summary().await.unwrap(), Some(summary));
    }

    #[tokio::test]
    async fn drops_invalid_legacy_summary() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("account_summary.bin");
        let legacy_path = tempdir.path().join("account_summary.json");
        tokio::fs::write(&legacy_path, "{").await.unwrap();
        let storage = storage(&path).with_legacy_plaintext(&legacy_path);

        assert_eq!(storage.load_summary().await.unwrap(), None);
        assert!(!legacy_path.exists());
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn prefers_stored_summary_over_legacy() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("account_summary.bin");
        let legacy_path = tempdir.path().join("account_summary.json");
        let storage = storage(&path).with_legacy_plaintext(&legacy_path);
        storage.store_summary(&summary()).await.unwrap();
        tokio::fs::write(&legacy_path, LEGACY_SUMMARY)
            .await
            .unwrap();

        assert_eq!(storage.load_summary().await.unwrap(), Some(summary()));
    }

    #[tokio::test]
    async fn remove_deletes_legacy_summary() {
        let tempdir = tempfile::tempdir().unwrap();
        let legacy_path = tempdir.path().join("account_summary.json");
        tokio::fs::write(&legacy_path, LEGACY_SUMMARY)
            .await
            .unwrap();
        let storage = storage(&tempdir.path().join("account_summary.bin"))
            .with_legacy_plaintext(&legacy_path);

        storage.remove_summary().await.unwrap();

        assert!(!legacy_path.exists());
        assert_eq!(storage.load_summary().await.unwrap(), None);
    }

    #[tokio::test]
    async fn load_returns_none_if_file_does_not_exist() {
        let tempdir = tempfile::tempdir().unwrap();
        let storage = storage(&tempdir.path().join("account_summary.bin"));

        assert_eq!(storage.load_summary().await.unwrap(), None);
    }

    #[tokio::test]
    async fn store_load_and_remove() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("nested/account_summary.bin");
        let storage = storage(&path);
        let summary = summary();

        storage.store_summary(&summary).await.unwrap();
        assert_eq!(storage.load_summary().await.unwrap(), Some(summary));

        storage.remove_summary().await.unwrap();
        assert!(!path.exists());
        assert_eq!(storage.load_summary().await.unwrap(), None);
    }
}
