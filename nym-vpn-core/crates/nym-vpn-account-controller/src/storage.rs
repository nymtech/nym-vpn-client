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

use crate::error::Error;

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
}

pub(crate) struct VpnCredentialStorage {
    pub(crate) storage: nym_credential_storage::persistent_storage::PersistentStorage,
}

impl VpnCredentialStorage {
    pub(crate) async fn check_local_remaining_tickets(&self) -> Vec<(TicketType, u32)> {
        // TODO: remove unwrap
        let ticketbooks_info = self.storage.get_ticketbooks_info().await.unwrap();

        // For each ticketbook type, iterate over and check if we have enough tickets stored
        // locally
        let ticketbook_types = ticketbook_types();

        let mut request_zk_nym = Vec::new();
        for ticketbook_type in ticketbook_types.iter() {
            let available_tickets: u32 = ticketbooks_info
                .iter()
                .filter(|ticketbook| ticketbook.ticketbook_type == ticketbook_type.to_string())
                .map(|ticketbook| ticketbook.total_tickets - ticketbook.used_tickets)
                .sum();

            request_zk_nym.push((*ticketbook_type, available_tickets));
        }
        request_zk_nym
    }

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
        let ticketbooks_info = self.storage.get_ticketbooks_info().await?;
        tracing::info!("Ticketbooks stored: {}", ticketbooks_info.len());
        for ticketbook in ticketbooks_info {
            tracing::info!("Ticketbook id: {}", ticketbook.id);
        }

        let pending_ticketbooks = self.storage.get_pending_ticketbooks().await?;
        for pending in pending_ticketbooks {
            tracing::info!("Pending ticketbook id: {}", pending.pending_id);
        }
        Ok(())
    }
}

// TODO: add #[derive(EnumIter)] to TicketType so we can iterate over it directly.
fn ticketbook_types() -> [TicketType; 4] {
    [
        TicketType::V1MixnetEntry,
        TicketType::V1MixnetExit,
        TicketType::V1WireguardEntry,
        TicketType::V1WireguardExit,
    ]
}
