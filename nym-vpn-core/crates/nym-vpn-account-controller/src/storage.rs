// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_compact_ecash::VerificationKeyAuth;
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
    IssuedTicketBook,
};
use nym_credentials_interface::TicketType;
use nym_sdk::mixnet::CredentialStorage;
use nym_vpn_api_client::types::{Device, VpnApiAccount};
use nym_vpn_store::{keys::KeyStore, mnemonic::MnemonicStorage, VpnStorage};

use crate::{error::Error, AvailableTicketbooks};

// If we go below this threshold, we should request more tickets
// TODO: I picked a random number, check what is should be. Or if we can express this in terms of
// data/bandwidth.
const TICKET_NUMBER_THRESHOLD: u64 = 30;

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

pub(crate) struct VpnCredentialStorage {
    pub(crate) storage: nym_credential_storage::persistent_storage::PersistentStorage,
}

impl VpnCredentialStorage {
    pub(crate) async fn insert_issued_ticketbook(
        &self,
        ticketbook: &IssuedTicketBook,
    ) -> Result<(), Error> {
        self.storage
            .insert_issued_ticketbook(ticketbook)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_master_verification_key(
        &self,
        key: &EpochVerificationKey,
    ) -> Result<(), Error> {
        self.storage
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
            .get_master_verification_key(epoch_id)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_coin_index_signatures(
        &self,
        signatures: &AggregatedCoinIndicesSignatures,
    ) -> Result<(), Error> {
        self.storage
            .insert_coin_index_signatures(signatures)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn insert_expiration_date_signatures(
        &self,
        signatures: &AggregatedExpirationDateSignatures,
    ) -> Result<(), Error> {
        self.storage
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

        let pending_ticketbooks = self.storage.get_pending_ticketbooks().await?;
        for pending in pending_ticketbooks {
            tracing::info!("Pending ticketbook id: {}", pending.pending_id);
        }
        Ok(())
    }

    pub(crate) async fn get_available_ticketbooks(&self) -> Result<AvailableTicketbooks, Error> {
        let ticketbooks_info = self.storage.get_ticketbooks_info().await?;
        AvailableTicketbooks::try_from(ticketbooks_info)
    }

    pub(crate) async fn check_ticket_types_running_low(&self) -> Result<Vec<TicketType>, Error> {
        let available_ticketbooks = self.get_available_ticketbooks().await?;

        let ticketbook_types_running_low = crate::ticketbooks::ticketbook_types()
            .into_iter()
            .filter(|ticket_type| {
                tracing::info!(
                    "Remaining unexpired tickets for {ticket_type}: {}",
                    available_ticketbooks.remaining_tickets(*ticket_type)
                );
                available_ticketbooks.remaining_tickets(*ticket_type) < TICKET_NUMBER_THRESHOLD
            })
            .collect();

        Ok(ticketbook_types_running_low)
    }

    pub(crate) async fn check_available_tickets_to_connect(&self) -> Result<bool, Error> {
        let ticket_types_running_low = self.check_ticket_types_running_low().await?;
        // If no ticket types are running low, we are ready to try to connect
        Ok(ticket_types_running_low.is_empty())
    }
}
