// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use bincode::Options;
use nym_compact_ecash::VerificationKeyAuth;
use nym_credential_storage::persistent_storage::PersistentStorage as PersistentCredentialStorage;
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
    IssuedTicketBook,
};
use nym_credentials_interface::{
    AnnotatedCoinIndexSignature, AnnotatedExpirationDateSignature, RequestInfo, TicketType,
};
use nym_sdk::mixnet::{CredentialStorage, StoragePaths};
use nym_vpn_api_client::types::{Device, VpnApiAccount};
use nym_vpn_store::{mnemonic::Mnemonic, VpnStorage};
use serde::{Deserialize, Serialize};
use sqlx::{ConnectOptions, FromRow};
use time::Date;
use tracing::log::LevelFilter;

use crate::{error::Error, AvailableTicketbooks};

#[derive(Debug, Clone)]
pub(crate) struct AccountStorage<S>
where
    S: VpnStorage,
{
    storage: Arc<tokio::sync::Mutex<S>>,
}

impl<S> AccountStorage<S>
where
    S: VpnStorage,
{
    pub(crate) fn from(storage: Arc<tokio::sync::Mutex<S>>) -> Self {
        Self { storage }
    }

    pub(crate) async fn store_account(&self, mnemonic: Mnemonic) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .store_mnemonic(mnemonic)
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_account(&self) -> Result<VpnApiAccount, Error> {
        self.storage
            .lock()
            .await
            .load_mnemonic()
            .await
            .map(VpnApiAccount::from)
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn remove_account(&self) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .remove_mnemonic()
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_account_id(&self) -> Result<String, Error> {
        self.load_account().await.map(|account| account.id())
    }

    pub(crate) async fn init_keys(&self) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .init_keys(None)
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_device_keys(&self) -> Result<Device, Error> {
        self.storage
            .lock()
            .await
            .load_keys()
            .await
            .map(|keys| Device::from(keys.device_keypair()))
            .inspect(|device| {
                tracing::debug!("Loading device keys: {}", device.identity_key());
            })
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_device_id(&self) -> Result<String, Error> {
        self.load_device_keys()
            .await
            .map(|device| device.identity_key().to_string())
    }

    pub(crate) async fn remove_device_keys(&self) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .remove_keys()
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }
}

#[derive(Clone)]
pub(crate) struct VpnCredentialStorage {
    data_dir: PathBuf,

    // TODO: remove Arc<Mutex>?
    storage: Arc<tokio::sync::Mutex<PersistentCredentialStorage>>,

    pub(crate) pending_requests: PendingCredentialRequestsStorage,
}

impl VpnCredentialStorage {
    pub(crate) async fn setup_from_path<P: AsRef<Path>>(data_dir: P) -> Result<Self, Error> {
        let storage_paths =
            StoragePaths::new_from_dir(data_dir.as_ref()).map_err(Error::StoragePaths)?;
        let storage = storage_paths
            .persistent_credential_storage()
            .await
            .map_err(Error::SetupCredentialStorage)?;
        let storage = Arc::new(tokio::sync::Mutex::new(storage));

        let pending_requests = PendingCredentialRequestsStorage::init(
            data_dir.as_ref().join("pending_credential_requests.db"),
        )
        .await
        .map_err(Error::SetupPendingCredentialRequestsStorage)?;

        Ok(Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            storage,
            pending_requests,
        })
    }

    pub(crate) async fn reset(&mut self) -> Result<(), Error> {
        let mut guard = self.storage.lock().await;

        // First we close the storage to ensure that all files are closed
        guard.close().await;

        // Calling close on the storage should be enough to ensure that all files are closed
        // but just to be sure we wait a bit
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Then we remove the credential database file
        let storage_paths =
            StoragePaths::new_from_dir(&self.data_dir).map_err(Error::StoragePaths)?;

        std::fs::remove_file(&storage_paths.credential_database_path)
            .inspect_err(|err| {
                tracing::error!("Failed to remove file: {err:?}");
            })
            .ok();

        // Finally we recreate the storage
        *guard = storage_paths
            .persistent_credential_storage()
            .await
            .map_err(Error::SetupCredentialStorage)?;

        // WIP(JON): reset pending requests

        Ok(())
    }

    pub(crate) async fn insert_issued_ticketbook(
        &self,
        ticketbook: &IssuedTicketBook,
    ) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .insert_issued_ticketbook(ticketbook)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_master_verification_key(
        &self,
        key: &EpochVerificationKey,
    ) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .insert_master_verification_key(key)
            .await
            .map_err(Error::from)
    }

    #[allow(unused)]
    pub(crate) async fn get_master_verification_key(
        &self,
        epoch_id: u64,
    ) -> Result<Option<VerificationKeyAuth>, Error> {
        self.storage
            .lock()
            .await
            .get_master_verification_key(epoch_id)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_coin_index_signatures(
        &self,
        signatures: &AggregatedCoinIndicesSignatures,
    ) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .insert_coin_index_signatures(signatures)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn get_coin_index_signatures(
        &self,
        epoch_id: u64,
    ) -> Result<Option<Vec<AnnotatedCoinIndexSignature>>, Error> {
        self.storage
            .lock()
            .await
            .get_coin_index_signatures(epoch_id)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_expiration_date_signatures(
        &self,
        signatures: &AggregatedExpirationDateSignatures,
    ) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .insert_expiration_date_signatures(signatures)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn get_expiration_date_signatures(
        &self,
        expiration_date: Date,
    ) -> Result<Option<Vec<AnnotatedExpirationDateSignature>>, Error> {
        self.storage
            .lock()
            .await
            .get_expiration_date_signatures(expiration_date)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn print_info(&self) -> Result<(), Error> {
        let ticketbooks_info = self.get_available_ticketbooks().await?;
        tracing::info!("Ticketbooks stored: {}", ticketbooks_info.len());
        for ticketbook in ticketbooks_info {
            tracing::info!("Ticketbook: {ticketbook}");
        }

        let pending_ticketbooks = self.storage.lock().await.get_pending_ticketbooks().await?;
        for pending in pending_ticketbooks {
            tracing::info!("Pending ticketbook id: {}", pending.pending_id);
        }
        Ok(())
    }

    pub(crate) async fn get_available_ticketbooks(&self) -> Result<AvailableTicketbooks, Error> {
        let ticketbooks_info = self.storage.lock().await.get_ticketbooks_info().await?;
        AvailableTicketbooks::try_from(ticketbooks_info)
    }

    pub(crate) async fn check_ticket_types_running_low(&self) -> Result<Vec<TicketType>, Error> {
        self.get_available_ticketbooks()
            .await
            .map(|ticketbooks| ticketbooks.ticket_types_running_low())
    }

    pub(crate) async fn get_pending_requests(
        &self,
    ) -> Result<Vec<PendingCredentialRequestStored>, Error> {
        self.pending_requests
            .get_pending_requests()
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn get_pending_request_by_id(
        &self,
        id: &str,
    ) -> Result<Option<PendingCredentialRequestStored>, Error> {
        self.pending_requests
            .get_pending_request_by_id(id)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_pending_request(
        &self,
        id: &str,
        expiration_date: Date,
        request_info: &RequestInfo,
    ) -> Result<(), Error> {
        self.pending_requests
            .insert_pending_request(id, expiration_date, request_info)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn remove_pending_request(&self, id: &str) -> Result<(), Error> {
        self.pending_requests
            .remove_pending_request(id)
            .await
            .map_err(Error::from)
    }
}

#[derive(Clone)]
struct SqliteZkNymRequestsStorageManager {
    connection_pool: sqlx::SqlitePool,
}

// Functions that does the queries
impl SqliteZkNymRequestsStorageManager {
    async fn get_pending_requests(
        &self,
    ) -> Result<Vec<PendingCredentialRequestStored>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM pending_zk_nym_requests")
            .fetch_all(&self.connection_pool)
            .await
    }

    async fn get_pending_request_by_id(
        &self,
        id: &str,
    ) -> Result<Option<PendingCredentialRequestStored>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM pending_zk_nym_requests WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.connection_pool)
            .await
    }

    async fn insert_pending_request(
        &self,
        id: &str,
        expiration_date: Date,
        request_info: &[u8],
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO pending_zk_nym_requests (id, expiration_date, request_info) VALUES (?, ?, ?)",
            id,
            expiration_date,
            request_info,
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    async fn remove_pending_request(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM pending_zk_nym_requests WHERE id = ?", id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PendingCredentialRequestsStorageError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migrate error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),
}

#[derive(Clone)]
pub(crate) struct PendingCredentialRequestsStorage {
    storage_manager: SqliteZkNymRequestsStorageManager,
}

impl PendingCredentialRequestsStorage {
    async fn init<P: AsRef<Path>>(
        database_path: P,
    ) -> Result<Self, PendingCredentialRequestsStorageError> {
        tracing::info!(
            "Setting up pending credential requests storage: {:?}",
            database_path.as_ref().as_os_str()
        );

        let opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true)
            .log_statements(LevelFilter::Info);

        let connection_pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(opts)
            .await?;

        sqlx::migrate!("./migrations").run(&connection_pool).await?;

        Ok(Self {
            storage_manager: SqliteZkNymRequestsStorageManager { connection_pool },
        })
    }

    async fn insert_pending_request(
        &self,
        id: &str,
        expiration_date: Date,
        request_info: &RequestInfo,
    ) -> Result<(), PendingCredentialRequestsStorageError> {
        let request_info = request_info_to_bytes(request_info)?;
        self.storage_manager
            .insert_pending_request(id, expiration_date, &request_info)
            .await
            .map_err(Into::into)
    }

    async fn get_pending_requests(
        &self,
    ) -> Result<Vec<PendingCredentialRequestStored>, PendingCredentialRequestsStorageError> {
        self.storage_manager
            .get_pending_requests()
            .await
            .map_err(Into::into)
    }

    async fn get_pending_request_by_id(
        &self,
        id: &str,
    ) -> Result<Option<PendingCredentialRequestStored>, PendingCredentialRequestsStorageError> {
        self.storage_manager
            .get_pending_request_by_id(id)
            .await
            .map_err(Into::into)
    }

    async fn remove_pending_request(
        &self,
        id: &str,
    ) -> Result<(), PendingCredentialRequestsStorageError> {
        self.storage_manager
            .remove_pending_request(id)
            .await
            .map_err(Into::into)
    }
}

// MODELS

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub(crate) struct PendingCredentialRequestStored {
    // WIP: remove pub
    pub(crate) id: String,
    // WIP: remove pub
    pub(crate) expiration_date: Date,
    // WIP: remove pub
    pub(crate) request_info: Vec<u8>,
}

pub(crate) fn request_info_to_bytes(request_info: &RequestInfo) -> Result<Vec<u8>, bincode::Error> {
    binary_serialiser().serialize(request_info)
}

pub(crate) fn request_info_from_bytes(bytes: &[u8]) -> Result<RequestInfo, bincode::Error> {
    binary_serialiser().deserialize(bytes)
}

fn binary_serialiser() -> impl bincode::Options {
    use bincode::Options;
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_varint_encoding()
}
