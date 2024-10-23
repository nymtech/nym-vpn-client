// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use nym_compact_ecash::{Base58 as _, BlindedSignature, VerificationKeyAuth};
use nym_config::defaults::NymNetworkDetails;
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
    IssuedTicketBook,
};
use nym_credentials_interface::{RequestInfo, TicketType};
use nym_ecash_time::EcashTime;
use nym_http_api_client::UserAgent;
use nym_sdk::mixnet::CredentialStorage;
use nym_vpn_api_client::{
    response::{NymErrorResponse, NymVpnZkNym, NymVpnZkNymStatus},
    types::{Device, VpnApiAccount},
    HttpClientError, VpnApiClientError,
};
use nym_vpn_store::{keys::KeyStore, mnemonic::MnemonicStorage, VpnStorage};
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::{JoinError, JoinSet},
    time::timeout,
};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    ecash_client::VpnEcashApiClient,
    error::Error,
    shared_state::{
        AccountState, DeviceState, MnemonicState, ReadyToRegisterDevice, SharedAccountState,
        SubscriptionState,
    },
};

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
            .inspect(|account| tracing::info!("Loading account id: {}", account.id()))
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
    pub(crate) async fn load_device_keys_from_storage(&self) -> Result<Device, <S as KeyStore>::StorageError> {
        self.storage
            .lock()
            .await
            .load_keys()
            .await
            .map(|keys| Device::from(keys.device_keypair()))
            .inspect(|device| {
                tracing::info!("Loading device keys: {}", device.identity_key());
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
