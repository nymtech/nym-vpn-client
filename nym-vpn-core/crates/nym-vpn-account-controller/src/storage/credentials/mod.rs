// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod pending_credential_requests;

pub use pending_credential_requests::{
    error::PendingCredentialRequestsStorageError, models::PendingCredentialRequest,
};

use pending_credential_requests::PendingCredentialRequestsStorage;

use std::path::{Path, PathBuf};

use nym_compact_ecash::VerificationKeyAuth;
use nym_credential_storage::persistent_storage::PersistentStorage as PersistentCredentialStorage;
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
    IssuedTicketBook,
};
use nym_credentials_interface::{
    AnnotatedCoinIndexSignature, AnnotatedExpirationDateSignature, TicketType,
};
use nym_sdk::mixnet::{CredentialStorage, StoragePaths};
use time::Date;

use crate::{error::Error, AvailableTicketbooks};

#[derive(Clone)]
pub(crate) struct VpnCredentialStorage {
    data_dir: PathBuf,
    credential_storage: PersistentCredentialStorage,
    pending_requests_storage: PendingCredentialRequestsStorage,
}

impl VpnCredentialStorage {
    pub(crate) async fn setup_from_path<P: AsRef<Path>>(data_dir: P) -> Result<Self, Error> {
        let storage_paths =
            StoragePaths::new_from_dir(data_dir.as_ref()).map_err(Error::StoragePaths)?;
        let storage = storage_paths
            .persistent_credential_storage()
            .await
            .map_err(Error::SetupCredentialStorage)?;

        let pending_requests = PendingCredentialRequestsStorage::init(
            data_dir.as_ref().join("pending_credential_requests.db"),
        )
        .await
        .map_err(Error::SetupPendingCredentialRequestsStorage)?;

        Ok(Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            credential_storage: storage,
            pending_requests_storage: pending_requests,
        })
    }

    pub(crate) async fn reset(&mut self) -> Result<(), Error> {
        self.reset_credential_storage().await?;
        self.reset_pending_request_storage().await?;
        Ok(())
    }

    async fn reset_credential_storage(&mut self) -> Result<(), Error> {
        tracing::info!("Resetting credential storage");

        // First we close the storage to ensure that all files are closed
        tracing::debug!("Closing credential storage");
        self.credential_storage.close().await;

        // Calling close on the storage should be enough to ensure that all files are closed
        // but just to be sure we wait a bit
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Then we remove the credential database file
        let storage_paths =
            StoragePaths::new_from_dir(&self.data_dir).map_err(Error::StoragePaths)?;

        tracing::debug!("Removing credential storage file");
        for path in storage_paths.credential_database_paths() {
            match std::fs::remove_file(&path) {
                Ok(_) => tracing::trace!("Removed file: {}", path.display()),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    tracing::trace!("File not found, skipping: {}", path.display())
                }
                Err(err) => {
                    tracing::error!("Failed to remove file {}: {err}", path.display());
                    return Err(Error::RemoveCredentialStorage(err));
                }
            }
        }

        // Finally we recreate the storage
        tracing::debug!("Recreating credential storage");
        self.credential_storage = storage_paths
            .persistent_credential_storage()
            .await
            .map_err(Error::SetupCredentialStorage)?;

        tracing::info!("Credential storage reset completed");

        Ok(())
    }

    async fn reset_pending_request_storage(&mut self) -> Result<(), Error> {
        tracing::info!("Resetting pending request storage");
        self.pending_requests_storage.reset().await?;
        tracing::info!("Pending request storage reset completed");
        Ok(())
    }

    pub(crate) async fn insert_issued_ticketbook(
        &self,
        ticketbook: &IssuedTicketBook,
    ) -> Result<(), Error> {
        self.credential_storage
            .insert_issued_ticketbook(ticketbook)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_master_verification_key(
        &self,
        key: &EpochVerificationKey,
    ) -> Result<(), Error> {
        self.credential_storage
            .insert_master_verification_key(key)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn get_master_verification_key(
        &self,
        epoch_id: u64,
    ) -> Result<Option<VerificationKeyAuth>, Error> {
        self.credential_storage
            .get_master_verification_key(epoch_id)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_coin_index_signatures(
        &self,
        signatures: &AggregatedCoinIndicesSignatures,
    ) -> Result<(), Error> {
        self.credential_storage
            .insert_coin_index_signatures(signatures)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn get_coin_index_signatures(
        &self,
        epoch_id: u64,
    ) -> Result<Option<Vec<AnnotatedCoinIndexSignature>>, Error> {
        self.credential_storage
            .get_coin_index_signatures(epoch_id)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_expiration_date_signatures(
        &self,
        signatures: &AggregatedExpirationDateSignatures,
    ) -> Result<(), Error> {
        self.credential_storage
            .insert_expiration_date_signatures(signatures)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn get_expiration_date_signatures(
        &self,
        expiration_date: Date,
    ) -> Result<Option<Vec<AnnotatedExpirationDateSignature>>, Error> {
        self.credential_storage
            .get_expiration_date_signatures(expiration_date)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn print_info(&self) -> Result<(), Error> {
        let ticketbooks_info = self.get_available_ticketbooks().await?;
        let num_ticketbooks = ticketbooks_info.len_not_expired();
        let num_total_ticketbooks = ticketbooks_info.len();
        tracing::info!("Ticketbooks stored: {num_ticketbooks}");
        tracing::debug!("Total ticketbooks stored: {num_total_ticketbooks}");
        for ticketbook in ticketbooks_info {
            if ticketbook.has_expired() {
                tracing::debug!("Ticketbook: {ticketbook}");
            } else {
                tracing::info!("Ticketbook: {ticketbook}");
            }
        }

        let pending_ticketbooks = self.credential_storage.get_pending_ticketbooks().await?;
        for pending in pending_ticketbooks {
            tracing::info!("Pending ticketbook id: {}", pending.pending_id);
        }
        Ok(())
    }

    pub(crate) async fn get_available_ticketbooks(&self) -> Result<AvailableTicketbooks, Error> {
        let ticketbooks_info = self.credential_storage.get_ticketbooks_info().await?;
        AvailableTicketbooks::try_from(ticketbooks_info)
    }

    pub(crate) async fn get_ticket_types_running_low(&self) -> Result<Vec<TicketType>, Error> {
        self.get_available_ticketbooks()
            .await
            .map(|ticketbooks| ticketbooks.ticket_types_running_low())
    }

    pub(crate) async fn is_all_ticket_types_above_soft_threshold(&self) -> Result<bool, Error> {
        self.get_available_ticketbooks()
            .await
            .map(|ticketbooks| ticketbooks.is_all_ticket_types_above_soft_threshold())
    }

    #[allow(unused)]
    pub(crate) async fn get_pending_requests(
        &self,
    ) -> Result<Vec<PendingCredentialRequest>, Error> {
        self.pending_requests_storage
            .get_pending_requests()
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn get_pending_request_ids(&self) -> Result<Vec<String>, Error> {
        self.pending_requests_storage
            .get_pending_requests()
            .await
            .map(|requests| requests.into_iter().map(|r| r.id.clone()).collect())
            .map_err(Error::from)
    }

    pub(crate) async fn get_pending_request_by_id(
        &self,
        id: &str,
    ) -> Result<Option<PendingCredentialRequest>, Error> {
        self.pending_requests_storage
            .get_pending_request_by_id(id)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_pending_request(
        &self,
        pending_request: PendingCredentialRequest,
    ) -> Result<(), Error> {
        tracing::debug!("Inserting pending request with id: {}", pending_request.id);
        self.pending_requests_storage
            .insert_pending_request(pending_request)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn remove_pending_request(&self, id: &str) -> Result<(), Error> {
        tracing::debug!("Removing pending request with id: {}", id);
        self.pending_requests_storage
            .remove_pending_request(id)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn clean_up_stale_requests(&self) -> Result<(), Error> {
        self.pending_requests_storage
            .clean_up_stale_requests()
            .await
            .map_err(Error::from)
    }
}
