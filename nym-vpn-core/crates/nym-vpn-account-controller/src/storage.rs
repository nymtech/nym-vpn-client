// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::Path, sync::Arc};

use nym_compact_ecash::VerificationKeyAuth;
use nym_credential_storage::persistent_storage::PersistentStorage as PersistentCredentialStorage;
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
    IssuedTicketBook,
};
use nym_credentials_interface::TicketType;
use nym_sdk::mixnet::CredentialStorage;
use nym_vpn_api_client::types::{Device, VpnApiAccount};
use nym_vpn_store::{keys::KeyStore, mnemonic::MnemonicStorage, VpnStorage};

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

    // Load account and keep the error type
    pub(crate) async fn load_account_from_storage(
        &self,
    ) -> Result<VpnApiAccount, <S as MnemonicStorage>::StorageError> {
        self.storage
            .lock()
            .await
            .load_mnemonic()
            .await
            .map(VpnApiAccount::from)
            .inspect(|account| tracing::debug!("Loading account id: {}", account.id()))
    }

    // Convenience function to load account and box the error
    pub(crate) async fn load_account(&self) -> Result<VpnApiAccount, Error> {
        self.load_account_from_storage()
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_account_id(&self) -> Result<String, Error> {
        self.load_account().await.map(|account| account.id())
    }

    // Load device keys and keep the error type
    pub(crate) async fn load_device_keys_from_storage(
        &self,
    ) -> Result<Device, <S as KeyStore>::StorageError> {
        self.storage
            .lock()
            .await
            .load_keys()
            .await
            .map(|keys| Device::from(keys.device_keypair()))
            .inspect(|device| {
                tracing::debug!("Loading device keys: {}", device.identity_key());
            })
    }

    // Convenience function to load device keys and box the error
    pub(crate) async fn load_device_keys(&self) -> Result<Device, Error> {
        self.load_device_keys_from_storage()
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_device_id(&self) -> Result<String, Error> {
        self.load_device_keys()
            .await
            .map(|device| device.identity_key().to_string())
    }
}

#[derive(Clone)]
pub(crate) struct VpnCredentialStorage {
    pub(crate) storage: Arc<tokio::sync::Mutex<PersistentCredentialStorage>>,
}

impl VpnCredentialStorage {
    pub(crate) async fn setup_from_path<P: AsRef<Path>>(data_dir: P) -> Result<Self, Error> {
        let storage_paths =
            nym_sdk::mixnet::StoragePaths::new_from_dir(data_dir).map_err(Error::StoragePaths)?;
        let storage = storage_paths
            .persistent_credential_storage()
            .await
            .map_err(Error::SetupCredentialStorage)?;
        let storage = Arc::new(tokio::sync::Mutex::new(storage));
        Ok(Self::from(storage))
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
}

impl From<Arc<tokio::sync::Mutex<PersistentCredentialStorage>>> for VpnCredentialStorage {
    fn from(storage: Arc<tokio::sync::Mutex<PersistentCredentialStorage>>) -> Self {
        Self { storage }
    }
}
